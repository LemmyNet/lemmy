use crate::{
  apub::{
    activities::{generate_activity_id, send_activity_to_community},
    check_actor_domain,
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    fetch_webfinger_url,
    fetcher::{
      get_or_fetch_and_insert_comment,
      get_or_fetch_and_insert_post,
      get_or_fetch_and_upsert_user,
    },
    ActorType,
    ApubLikeableType,
    ApubObjectType,
    FromApub,
    ToApub,
  },
  DbPool,
  LemmyContext,
};
use activitystreams::{
  activity::{
    kind::{CreateType, DeleteType, DislikeType, LikeType, RemoveType, UndoType, UpdateType},
    Create,
    Delete,
    Dislike,
    Like,
    Remove,
    Undo,
    Update,
  },
  base::AnyBase,
  link::Mention,
  object::{kind::NoteType, Note, Tombstone},
  prelude::*,
  public,
};
use actix_web::{body::Body, web, web::Path, HttpResponse};
use anyhow::Context;
use itertools::Itertools;
use lemmy_api_structs::blocking;
use lemmy_db::{
  comment::{Comment, CommentForm},
  community::Community,
  post::Post,
  user::User_,
  Crud,
};
use lemmy_utils::{
  location_info,
  utils::{convert_datetime, remove_slurs, scrape_text_for_mentions, MentionData},
  LemmyError,
};
use log::debug;
use serde::Deserialize;
use serde_json::Error;
use url::Url;

#[derive(Deserialize)]
pub struct CommentQuery {
  comment_id: String,
}

/// Return the post json over HTTP.
pub async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = info.comment_id.parse::<i32>()?;
  let comment = blocking(context.pool(), move |conn| Comment::read(conn, id)).await??;

  if !comment.deleted {
    Ok(create_apub_response(
      &comment.to_apub(context.pool()).await?,
    ))
  } else {
    Ok(create_apub_tombstone_response(&comment.to_tombstone()?))
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for Comment {
  type Response = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let mut comment = Note::new();

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    // Add a vector containing some important info to the "in_reply_to" field
    // [post_ap_id, Option(parent_comment_ap_id)]
    let mut in_reply_to_vec = vec![post.ap_id];

    if let Some(parent_id) = self.parent_id {
      let parent_comment = blocking(pool, move |conn| Comment::read(conn, parent_id)).await??;

      in_reply_to_vec.push(parent_comment.ap_id);
    }

    comment
      // Not needed when the Post is embedded in a collection (like for community outbox)
      .set_context(activitystreams::context())
      .set_id(Url::parse(&self.ap_id)?)
      .set_published(convert_datetime(self.published))
      .set_to(community.actor_id)
      .set_many_in_reply_tos(in_reply_to_vec)
      .set_content(self.content.to_owned())
      .set_attributed_to(creator.actor_id);

    if let Some(u) = self.updated {
      comment.set_updated(convert_datetime(u));
    }

    Ok(comment)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.ap_id, self.updated, NoteType::Note)
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for CommentForm {
  type ApubType = Note;

  /// Parse an ActivityPub note received from another instance into a Lemmy comment
  async fn from_apub(
    note: &Note,
    context: &LemmyContext,
    expected_domain: Option<Url>,
  ) -> Result<CommentForm, LemmyError> {
    let creator_actor_id = &note
      .attributed_to()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(creator_actor_id, context).await?;

    let mut in_reply_tos = note
      .in_reply_to()
      .as_ref()
      .context(location_info!())?
      .as_many()
      .context(location_info!())?
      .iter()
      .map(|i| i.as_xsd_any_uri().context(""));
    let post_ap_id = in_reply_tos.next().context(location_info!())??;

    // This post, or the parent comment might not yet exist on this server yet, fetch them.
    let post = get_or_fetch_and_insert_post(&post_ap_id, context).await?;

    // The 2nd item, if it exists, is the parent comment apub_id
    // For deeply nested comments, FromApub automatically gets called recursively
    let parent_id: Option<i32> = match in_reply_tos.next() {
      Some(parent_comment_uri) => {
        let parent_comment_ap_id = &parent_comment_uri?;
        let parent_comment =
          get_or_fetch_and_insert_comment(&parent_comment_ap_id, context).await?;

        Some(parent_comment.id)
      }
      None => None,
    };
    let content = note
      .content()
      .context(location_info!())?
      .as_single_xsd_string()
      .context(location_info!())?
      .to_string();
    let content_slurs_removed = remove_slurs(&content);

    Ok(CommentForm {
      creator_id: creator.id,
      post_id: post.id,
      parent_id,
      content: content_slurs_removed,
      removed: None,
      read: None,
      published: note.published().map(|u| u.to_owned().naive_local()),
      updated: note.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      ap_id: Some(check_actor_domain(note, expected_domain)?),
      local: false,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObjectType for Comment {
  /// Send out information about a newly created comment, to the followers of the community.
  async fn send_create(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let maa = collect_non_local_mentions_and_addresses(&self.content, &community, context).await?;

    let mut create = Create::new(creator.actor_id.to_owned(), note.into_any_base()?);
    create
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(CreateType::Create)?)
      .set_to(public())
      .set_many_ccs(maa.addressed_ccs.to_owned())
      // Set the mention tags
      .set_many_tags(maa.get_tags()?);

    send_activity_to_community(&creator, &community, maa.inboxes, create, context).await?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  async fn send_update(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let maa = collect_non_local_mentions_and_addresses(&self.content, &community, context).await?;

    let mut update = Update::new(creator.actor_id.to_owned(), note.into_any_base()?);
    update
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UpdateType::Update)?)
      .set_to(public())
      .set_many_ccs(maa.addressed_ccs.to_owned())
      // Set the mention tags
      .set_many_tags(maa.get_tags()?);

    send_activity_to_community(&creator, &community, maa.inboxes, update, context).await?;
    Ok(())
  }

  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), note.into_any_base()?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      delete,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    // Generate a fake delete activity, with the correct object
    let mut delete = Delete::new(creator.actor_id.to_owned(), note.into_any_base()?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      undo,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), note.into_any_base()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &mod_,
      &community,
      vec![community.get_shared_inbox_url()?],
      remove,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    // Generate a fake delete activity, with the correct object
    let mut remove = Remove::new(mod_.actor_id.to_owned(), note.into_any_base()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(mod_.actor_id.to_owned(), remove.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &mod_,
      &community,
      vec![community.get_shared_inbox_url()?],
      undo,
      context,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ApubLikeableType for Comment {
  async fn send_like(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut like = Like::new(creator.actor_id.to_owned(), note.into_any_base()?);
    like
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      like,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_dislike(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut dislike = Dislike::new(creator.actor_id.to_owned(), note.into_any_base()?);
    dislike
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DislikeType::Dislike)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      dislike,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_like(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut like = Like::new(creator.actor_id.to_owned(), note.into_any_base()?);
    like
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DislikeType::Dislike)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), like.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      undo,
      context,
    )
    .await?;
    Ok(())
  }
}

struct MentionsAndAddresses {
  addressed_ccs: Vec<Url>,
  inboxes: Vec<Url>,
  tags: Vec<Mention>,
}

impl MentionsAndAddresses {
  fn get_tags(&self) -> Result<Vec<AnyBase>, Error> {
    self
      .tags
      .iter()
      .map(|t| t.to_owned().into_any_base())
      .collect::<Result<Vec<AnyBase>, Error>>()
  }
}

/// This takes a comment, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the users / addresses that go in the cc field.
async fn collect_non_local_mentions_and_addresses(
  content: &str,
  community: &Community,
  context: &LemmyContext,
) -> Result<MentionsAndAddresses, LemmyError> {
  let mut addressed_ccs = vec![community.get_followers_url()?];

  // Add the mention tag
  let mut tags = Vec::new();

  // Get the inboxes for any mentions
  let mentions = scrape_text_for_mentions(&content)
    .into_iter()
    // Filter only the non-local ones
    .filter(|m| !m.is_local())
    .collect::<Vec<MentionData>>();

  let mut mention_inboxes: Vec<Url> = Vec::new();
  for mention in &mentions {
    // TODO should it be fetching it every time?
    if let Ok(actor_id) = fetch_webfinger_url(mention, context.client()).await {
      debug!("mention actor_id: {}", actor_id);
      addressed_ccs.push(actor_id.to_owned().to_string().parse()?);

      let mention_user = get_or_fetch_and_upsert_user(&actor_id, context).await?;
      let shared_inbox = mention_user.get_shared_inbox_url()?;

      mention_inboxes.push(shared_inbox);
      let mut mention_tag = Mention::new();
      mention_tag.set_href(actor_id).set_name(mention.full_name());
      tags.push(mention_tag);
    }
  }

  let mut inboxes = vec![community.get_shared_inbox_url()?];
  inboxes.extend(mention_inboxes);
  inboxes = inboxes.into_iter().unique().collect();

  Ok(MentionsAndAddresses {
    addressed_ccs,
    inboxes,
    tags,
  })
}
