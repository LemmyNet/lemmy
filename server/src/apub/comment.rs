use crate::{
  apub::{
    activities::{populate_object_props, send_activity_to_community},
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    fetch_webfinger_url,
    fetcher::{
      get_or_fetch_and_insert_remote_comment,
      get_or_fetch_and_insert_remote_post,
      get_or_fetch_and_upsert_remote_user,
    },
    ActorType,
    ApubLikeableType,
    ApubObjectType,
    FromApub,
    ToApub,
  },
  blocking,
  routes::DbPoolParam,
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{Create, Delete, Dislike, Like, Remove, Undo, Update},
  context,
  link::Mention,
  object::{kind::NoteType, properties::ObjectProperties, Note},
};
use activitystreams_new::object::Tombstone;
use actix_web::{body::Body, client::Client, web::Path, HttpResponse};
use itertools::Itertools;
use lemmy_db::{
  comment::{Comment, CommentForm},
  community::Community,
  post::Post,
  user::User_,
  Crud,
};
use lemmy_utils::{convert_datetime, scrape_text_for_mentions, MentionData};
use log::debug;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommentQuery {
  comment_id: String,
}

/// Return the post json over HTTP.
pub async fn get_apub_comment(
  info: Path<CommentQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = info.comment_id.parse::<i32>()?;
  let comment = blocking(&db, move |conn| Comment::read(conn, id)).await??;

  if !comment.deleted {
    Ok(create_apub_response(&comment.to_apub(&db).await?))
  } else {
    Ok(create_apub_tombstone_response(&comment.to_tombstone()?))
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for Comment {
  type Response = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let mut comment = Note::default();
    let oprops: &mut ObjectProperties = comment.as_mut();

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

    oprops
      // Not needed when the Post is embedded in a collection (like for community outbox)
      .set_context_xsd_any_uri(context())?
      .set_id(self.ap_id.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_to_xsd_any_uri(community.actor_id)?
      .set_many_in_reply_to_xsd_any_uris(in_reply_to_vec)?
      .set_content_xsd_string(self.content.to_owned())?
      .set_attributed_to_xsd_any_uri(creator.actor_id)?;

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    Ok(comment)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      &self.ap_id,
      self.updated,
      NoteType.to_string(),
    )
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for CommentForm {
  type ApubType = Note;

  /// Parse an ActivityPub note received from another instance into a Lemmy comment
  async fn from_apub(
    note: &Note,
    client: &Client,
    pool: &DbPool,
  ) -> Result<CommentForm, LemmyError> {
    let oprops = &note.object_props;
    let creator_actor_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();

    let creator = get_or_fetch_and_upsert_remote_user(&creator_actor_id, client, pool).await?;

    let mut in_reply_tos = oprops.get_many_in_reply_to_xsd_any_uris().unwrap();
    let post_ap_id = in_reply_tos.next().unwrap().to_string();

    // This post, or the parent comment might not yet exist on this server yet, fetch them.
    let post = get_or_fetch_and_insert_remote_post(&post_ap_id, client, pool).await?;

    // The 2nd item, if it exists, is the parent comment apub_id
    // For deeply nested comments, FromApub automatically gets called recursively
    let parent_id: Option<i32> = match in_reply_tos.next() {
      Some(parent_comment_uri) => {
        let parent_comment_ap_id = &parent_comment_uri.to_string();
        let parent_comment =
          get_or_fetch_and_insert_remote_comment(&parent_comment_ap_id, client, pool).await?;

        Some(parent_comment.id)
      }
      None => None,
    };

    Ok(CommentForm {
      creator_id: creator.id,
      post_id: post.id,
      parent_id,
      content: oprops
        .get_content_xsd_string()
        .map(|c| c.to_string())
        .unwrap(),
      removed: None,
      read: None,
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      ap_id: oprops.get_id().unwrap().to_string(),
      local: false,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObjectType for Comment {
  /// Send out information about a newly created comment, to the followers of the community.
  async fn send_create(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let maa =
      collect_non_local_mentions_and_addresses(&self.content, &community, client, pool).await?;

    let id = format!("{}/create/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut create = Create::new();
    populate_object_props(&mut create.object_props, maa.addressed_ccs, &id)?;

    // Set the mention tags
    create.object_props.set_many_tag_base_boxes(maa.tags)?;

    create
      .create_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    send_activity_to_community(&creator, &community, maa.inboxes, create, client, pool).await?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  async fn send_update(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let maa =
      collect_non_local_mentions_and_addresses(&self.content, &community, client, pool).await?;

    let id = format!("{}/update/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut update = Update::new();
    populate_object_props(&mut update.object_props, maa.addressed_ccs, &id)?;

    // Set the mention tags
    update.object_props.set_many_tag_base_boxes(maa.tags)?;

    update
      .update_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    send_activity_to_community(&creator, &community, maa.inboxes, update, client, pool).await?;
    Ok(())
  }

  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut delete = Delete::default();

    populate_object_props(
      &mut delete.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()],
      delete,
      client,
      pool,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    // Generate a fake delete activity, with the correct object
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut delete = Delete::default();

    populate_object_props(
      &mut delete.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![community.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(delete)?;

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()],
      undo,
      client,
      pool,
    )
    .await?;
    Ok(())
  }

  async fn send_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let id = format!("{}/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut remove = Remove::default();

    populate_object_props(
      &mut remove.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(note)?;

    send_activity_to_community(
      &mod_,
      &community,
      vec![community.get_shared_inbox_url()],
      remove,
      client,
      pool,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    // Generate a fake delete activity, with the correct object
    let id = format!("{}/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut remove = Remove::default();

    populate_object_props(
      &mut remove.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(note)?;

    // Undo that fake activity
    let undo_id = format!("{}/undo/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![community.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(remove)?;

    send_activity_to_community(
      &mod_,
      &community,
      vec![community.get_shared_inbox_url()],
      undo,
      client,
      pool,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ApubLikeableType for Comment {
  async fn send_like(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let id = format!("{}/like/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(
      &mut like.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()],
      like,
      client,
      pool,
    )
    .await?;
    Ok(())
  }

  async fn send_dislike(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let id = format!("{}/dislike/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut dislike = Dislike::new();
    populate_object_props(
      &mut dislike.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    dislike
      .dislike_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()],
      dislike,
      client,
      pool,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_like(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let id = format!("{}/dislike/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(
      &mut like.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/like/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![community.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(like)?;

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()],
      undo,
      client,
      pool,
    )
    .await?;
    Ok(())
  }
}

struct MentionsAndAddresses {
  addressed_ccs: Vec<String>,
  inboxes: Vec<String>,
  tags: Vec<Mention>,
}

/// This takes a comment, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the users / addresses that go in the cc field.
async fn collect_non_local_mentions_and_addresses(
  content: &str,
  community: &Community,
  client: &Client,
  pool: &DbPool,
) -> Result<MentionsAndAddresses, LemmyError> {
  let mut addressed_ccs = vec![community.get_followers_url()];

  // Add the mention tag
  let mut tags = Vec::new();

  // Get the inboxes for any mentions
  let mentions = scrape_text_for_mentions(&content)
    .into_iter()
    // Filter only the non-local ones
    .filter(|m| !m.is_local())
    .collect::<Vec<MentionData>>();

  let mut mention_inboxes = Vec::new();
  for mention in &mentions {
    // TODO should it be fetching it every time?
    if let Ok(actor_id) = fetch_webfinger_url(mention, client).await {
      debug!("mention actor_id: {}", actor_id);
      addressed_ccs.push(actor_id.to_owned());

      let mention_user = get_or_fetch_and_upsert_remote_user(&actor_id, client, pool).await?;
      let shared_inbox = mention_user.get_shared_inbox_url();

      mention_inboxes.push(shared_inbox);
      let mut mention_tag = Mention::new();
      mention_tag
        .link_props
        .set_href(actor_id)?
        .set_name_xsd_string(mention.full_name())?;
      tags.push(mention_tag);
    }
  }

  let mut inboxes = vec![community.get_shared_inbox_url()];
  inboxes.extend(mention_inboxes);
  inboxes = inboxes.into_iter().unique().collect();

  Ok(MentionsAndAddresses {
    addressed_ccs,
    inboxes,
    tags,
  })
}
