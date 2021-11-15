use crate::{
  fetcher::webfinger::WebfingerResponse,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{
  base::BaseExt,
  link::{LinkExt, Mention},
};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{object_id::ObjectId, traits::ActorType};
use lemmy_db_schema::{
  source::{comment::Comment, person::Person, post::Post},
  traits::Crud,
  DbPool,
};
use lemmy_utils::{
  request::{retry, RecvError},
  utils::{scrape_text_for_mentions, MentionData},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use log::debug;
use url::Url;

pub struct MentionsAndAddresses {
  pub ccs: Vec<Url>,
  pub tags: Vec<Mention>,
}

/// This takes a comment, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the persons / addresses that go in the cc field.
pub async fn collect_non_local_mentions(
  comment: &ApubComment,
  community_id: ObjectId<ApubCommunity>,
  context: &LemmyContext,
) -> Result<MentionsAndAddresses, LemmyError> {
  let parent_creator = get_comment_parent_creator(context.pool(), comment).await?;
  let mut addressed_ccs: Vec<Url> = vec![community_id.into(), parent_creator.actor_id()];

  // Add the mention tag
  let mut parent_creator_tag = Mention::new();
  parent_creator_tag
    .set_href(parent_creator.actor_id.clone().into())
    .set_name(format!(
      "@{}@{}",
      &parent_creator.name,
      &parent_creator.actor_id().domain().expect("has domain")
    ));
  let mut tags = vec![parent_creator_tag];

  // Get the person IDs for any mentions
  let mentions = scrape_text_for_mentions(&comment.content)
    .into_iter()
    // Filter only the non-local ones
    .filter(|m| !m.is_local(&context.settings().hostname))
    .collect::<Vec<MentionData>>();

  for mention in &mentions {
    // TODO should it be fetching it every time?
    if let Ok(actor_id) = fetch_webfinger_url(mention, context).await {
      let actor_id: ObjectId<ApubPerson> = ObjectId::new(actor_id);
      debug!("mention actor_id: {}", actor_id);
      addressed_ccs.push(actor_id.to_string().parse()?);

      let mut mention_tag = Mention::new();
      mention_tag
        .set_href(actor_id.into())
        .set_name(mention.full_name());
      tags.push(mention_tag);
    }
  }

  Ok(MentionsAndAddresses {
    ccs: addressed_ccs,
    tags,
  })
}

/// Returns the apub ID of the person this comment is responding to. Meaning, in case this is a
/// top-level comment, the creator of the post, otherwise the creator of the parent comment.
async fn get_comment_parent_creator(
  pool: &DbPool,
  comment: &Comment,
) -> Result<ApubPerson, LemmyError> {
  let parent_creator_id = if let Some(parent_comment_id) = comment.parent_id {
    let parent_comment =
      blocking(pool, move |conn| Comment::read(conn, parent_comment_id)).await??;
    parent_comment.creator_id
  } else {
    let parent_post_id = comment.post_id;
    let parent_post = blocking(pool, move |conn| Post::read(conn, parent_post_id)).await??;
    parent_post.creator_id
  };
  Ok(
    blocking(pool, move |conn| Person::read(conn, parent_creator_id))
      .await??
      .into(),
  )
}

/// Turns a person id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
/// using webfinger.
async fn fetch_webfinger_url(
  mention: &MentionData,
  context: &LemmyContext,
) -> Result<Url, LemmyError> {
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource=acct:{}@{}",
    context.settings().get_protocol_string(),
    mention.domain,
    mention.name,
    mention.domain
  );
  debug!("Fetching webfinger url: {}", &fetch_url);

  let response = retry(|| context.client().get(&fetch_url).send()).await?;

  let res: WebfingerResponse = response
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
    .ok_or_else(|| anyhow!("No href found.").into())
}
