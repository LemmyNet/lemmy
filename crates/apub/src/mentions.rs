use crate::{
  fetcher::webfinger::webfinger_resolve_actor,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
  ActorType,
};
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::link::MentionType;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::{comment::Comment, person::Person, post::Post},
  traits::Crud,
  utils::DbPool,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{scrape_text_for_mentions, MentionData},
};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MentionOrValue {
  Mention(Mention),
  Value(Value),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mention {
  pub href: Url,
  name: Option<String>,
  #[serde(rename = "type")]
  pub kind: MentionType,
}

pub struct MentionsAndAddresses {
  pub ccs: Vec<Url>,
  pub tags: Vec<MentionOrValue>,
}

/// This takes a comment, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the persons / addresses that go in the cc field.
#[tracing::instrument(skip(comment, community_id, context))]
pub async fn collect_non_local_mentions(
  comment: &ApubComment,
  community_id: ObjectId<ApubCommunity>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<MentionsAndAddresses, LemmyError> {
  let parent_creator = get_comment_parent_creator(context.pool(), comment).await?;
  let mut addressed_ccs: Vec<Url> = vec![community_id.into(), parent_creator.actor_id()];

  // Add the mention tag
  let parent_creator_tag = Mention {
    href: parent_creator.actor_id.clone().into(),
    name: Some(format!(
      "@{}@{}",
      &parent_creator.name,
      &parent_creator.actor_id().domain().expect("has domain")
    )),
    kind: MentionType::Mention,
  };
  let mut tags = vec![parent_creator_tag];

  // Get the person IDs for any mentions
  let mentions = scrape_text_for_mentions(&comment.content)
    .into_iter()
    // Filter only the non-local ones
    .filter(|m| !m.is_local(&context.settings().hostname))
    .collect::<Vec<MentionData>>();

  for mention in &mentions {
    // TODO should it be fetching it every time?
    let identifier = format!("{}@{}", mention.name, mention.domain);
    let actor_id =
      webfinger_resolve_actor::<ApubPerson>(&identifier, context, request_counter).await;
    if let Ok(actor_id) = actor_id {
      let actor_id: ObjectId<ApubPerson> = ObjectId::new(actor_id);
      addressed_ccs.push(actor_id.to_string().parse()?);

      let mention_tag = Mention {
        href: actor_id.into(),
        name: Some(mention.full_name()),
        kind: MentionType::Mention,
      };
      tags.push(mention_tag);
    }
  }

  let tags = tags.into_iter().map(MentionOrValue::Mention).collect();
  Ok(MentionsAndAddresses {
    ccs: addressed_ccs,
    tags,
  })
}

/// Returns the apub ID of the person this comment is responding to. Meaning, in case this is a
/// top-level comment, the creator of the post, otherwise the creator of the parent comment.
#[tracing::instrument(skip(pool, comment))]
async fn get_comment_parent_creator(
  pool: &DbPool,
  comment: &Comment,
) -> Result<ApubPerson, LemmyError> {
  let parent_creator_id = if let Some(parent_comment_id) = comment.parent_comment_id() {
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
