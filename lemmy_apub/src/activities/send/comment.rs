use crate::{
  activities::send::generate_activity_id,
  activity_queue::{send_comment_mentions, send_to_community},
  fetcher::get_or_fetch_and_upsert_user,
  ActorType,
  ApubLikeableType,
  ApubObjectType,
  ToApub,
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
  prelude::*,
  public,
};
use anyhow::anyhow;
use itertools::Itertools;
use lemmy_db::{comment::Comment, community::Community, post::Post, user::User_, Crud, DbPool};
use lemmy_structs::{blocking, WebFingerResponse};
use lemmy_utils::{
  request::{retry, RecvError},
  settings::Settings,
  utils::{scrape_text_for_mentions, MentionData},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use log::debug;
use reqwest::Client;
use serde_json::Error;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ApubObjectType for Comment {
  /// Send out information about a newly created comment, to the followers of the community and
  /// mentioned users.
  async fn send_create(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut maa = collect_non_local_mentions_and_addresses(&self.content, context).await?;
    let mut ccs = vec![community.actor_id()?];
    ccs.append(&mut maa.addressed_ccs);
    ccs.push(get_comment_parent_creator_id(context.pool(), &self).await?);

    let mut create = Create::new(creator.actor_id.to_owned(), note.into_any_base()?);
    create
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(CreateType::Create)?)
      .set_to(public())
      .set_many_ccs(ccs)
      // Set the mention tags
      .set_many_tags(maa.get_tags()?);

    send_to_community(create.clone(), &creator, &community, context).await?;
    send_comment_mentions(&creator, maa.inboxes, create, context).await?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community and mentioned
  /// users.
  async fn send_update(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut maa = collect_non_local_mentions_and_addresses(&self.content, context).await?;
    let mut ccs = vec![community.actor_id()?];
    ccs.append(&mut maa.addressed_ccs);
    ccs.push(get_comment_parent_creator_id(context.pool(), &self).await?);

    let mut update = Update::new(creator.actor_id.to_owned(), note.into_any_base()?);
    update
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UpdateType::Update)?)
      .set_to(public())
      .set_many_ccs(ccs)
      // Set the mention tags
      .set_many_tags(maa.get_tags()?);

    send_to_community(update.clone(), &creator, &community, context).await?;
    send_comment_mentions(&creator, maa.inboxes, update, context).await?;
    Ok(())
  }

  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(delete, &creator, &community, context).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    // Generate a fake delete activity, with the correct object
    let mut delete = Delete::new(creator.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(undo, &creator, &community, context).await?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(remove, &mod_, &community, context).await?;
    Ok(())
  }

  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    // Generate a fake delete activity, with the correct object
    let mut remove = Remove::new(mod_.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    // Undo that fake activity
    let mut undo = Undo::new(mod_.actor_id.to_owned(), remove.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(undo, &mod_, &community, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(like, &creator, &community, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(dislike, &creator, &community, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), like.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(undo, &creator, &community, context).await?;
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
  context: &LemmyContext,
) -> Result<MentionsAndAddresses, LemmyError> {
  let mut addressed_ccs = vec![];

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

      let mention_user = get_or_fetch_and_upsert_user(&actor_id, context, &mut 0).await?;
      let shared_inbox = mention_user.get_shared_inbox_url()?;

      mention_inboxes.push(shared_inbox);
      let mut mention_tag = Mention::new();
      mention_tag.set_href(actor_id).set_name(mention.full_name());
      tags.push(mention_tag);
    }
  }

  let inboxes = mention_inboxes.into_iter().unique().collect();

  Ok(MentionsAndAddresses {
    addressed_ccs,
    inboxes,
    tags,
  })
}

/// Returns the apub ID of the user this comment is responding to. Meaning, in case this is a
/// top-level comment, the creator of the post, otherwise the creator of the parent comment.
async fn get_comment_parent_creator_id(
  pool: &DbPool,
  comment: &Comment,
) -> Result<Url, LemmyError> {
  let parent_creator_id = if let Some(parent_comment_id) = comment.parent_id {
    let parent_comment =
      blocking(pool, move |conn| Comment::read(conn, parent_comment_id)).await??;
    parent_comment.creator_id
  } else {
    let parent_post_id = comment.post_id;
    let parent_post = blocking(pool, move |conn| Post::read(conn, parent_post_id)).await??;
    parent_post.creator_id
  };
  let parent_creator = blocking(pool, move |conn| User_::read(conn, parent_creator_id)).await??;
  Ok(parent_creator.actor_id()?)
}

/// Turns a user id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
/// using webfinger.
async fn fetch_webfinger_url(mention: &MentionData, client: &Client) -> Result<Url, LemmyError> {
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource=acct:{}@{}",
    Settings::get().get_protocol_string(),
    mention.domain,
    mention.name,
    mention.domain
  );
  debug!("Fetching webfinger url: {}", &fetch_url);

  let response = retry(|| client.get(&fetch_url).send()).await?;

  let res: WebFingerResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  let link = res
    .links
    .iter()
    .find(|l| l.type_.eq(&Some("application/activity+json".to_string())))
    .ok_or_else(|| anyhow!("No application/activity+json link found."))?;
  link
    .href
    .to_owned()
    .map(|u| Url::parse(&u))
    .transpose()?
    .ok_or_else(|| anyhow!("No href found.").into())
}
