use crate::{fetcher::person::get_or_fetch_and_upsert_person, ActorType};
use activitystreams::{
  base::BaseExt,
  link::{LinkExt, Mention},
};
use anyhow::anyhow;
use itertools::Itertools;
use lemmy_api_common::{blocking, send_local_notifs};
use lemmy_apub_lib::webfinger::WebfingerResponse;
use lemmy_db_queries::{Crud, DbPool};
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, person::Person, post::Post},
  LocalUserId,
};
use lemmy_utils::{
  request::{retry, RecvError},
  settings::structs::Settings,
  utils::{scrape_text_for_mentions, MentionData},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use log::debug;
use reqwest::Client;
use url::Url;

pub mod create_or_update;

async fn get_notif_recipients(
  actor: &Url,
  comment: &Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Vec<LocalUserId>, LemmyError> {
  let post_id = comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
  let actor = get_or_fetch_and_upsert_person(actor, context, request_counter).await?;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  // TODO: for compatibility with other projects, it would be much better to read this from cc or tags
  let mentions = scrape_text_for_mentions(&comment.content);
  send_local_notifs(mentions, comment.clone(), actor, post, context.pool(), true).await
}

pub struct MentionsAndAddresses {
  pub ccs: Vec<Url>,
  pub inboxes: Vec<Url>,
  pub tags: Vec<Mention>,
}

/// This takes a comment, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the persons / addresses that go in the cc field.
pub async fn collect_non_local_mentions(
  comment: &Comment,
  community: &Community,
  context: &LemmyContext,
) -> Result<MentionsAndAddresses, LemmyError> {
  let parent_creator = get_comment_parent_creator(context.pool(), comment).await?;
  let mut addressed_ccs = vec![community.actor_id(), parent_creator.actor_id()];
  // Note: dont include community inbox here, as we send to it separately with `send_to_community()`
  let mut inboxes = vec![parent_creator.get_shared_inbox_or_inbox_url()];

  // Add the mention tag
  let mut tags = Vec::new();

  // Get the person IDs for any mentions
  let mentions = scrape_text_for_mentions(&comment.content)
    .into_iter()
    // Filter only the non-local ones
    .filter(|m| !m.is_local())
    .collect::<Vec<MentionData>>();

  for mention in &mentions {
    // TODO should it be fetching it every time?
    if let Ok(actor_id) = fetch_webfinger_url(mention, context.client()).await {
      debug!("mention actor_id: {}", actor_id);
      addressed_ccs.push(actor_id.to_owned().to_string().parse()?);

      let mention_person = get_or_fetch_and_upsert_person(&actor_id, context, &mut 0).await?;
      inboxes.push(mention_person.get_shared_inbox_or_inbox_url());

      let mut mention_tag = Mention::new();
      mention_tag.set_href(actor_id).set_name(mention.full_name());
      tags.push(mention_tag);
    }
  }

  let inboxes = inboxes.into_iter().unique().collect();

  Ok(MentionsAndAddresses {
    ccs: addressed_ccs,
    inboxes,
    tags,
  })
}

/// Returns the apub ID of the person this comment is responding to. Meaning, in case this is a
/// top-level comment, the creator of the post, otherwise the creator of the parent comment.
async fn get_comment_parent_creator(
  pool: &DbPool,
  comment: &Comment,
) -> Result<Person, LemmyError> {
  let parent_creator_id = if let Some(parent_comment_id) = comment.parent_id {
    let parent_comment =
      blocking(pool, move |conn| Comment::read(conn, parent_comment_id)).await??;
    parent_comment.creator_id
  } else {
    let parent_post_id = comment.post_id;
    let parent_post = blocking(pool, move |conn| Post::read(conn, parent_post_id)).await??;
    parent_post.creator_id
  };
  Ok(blocking(pool, move |conn| Person::read(conn, parent_creator_id)).await??)
}

/// Turns a person id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
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
