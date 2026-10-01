use activitypub_federation::{config::Data, traits::Actor};
use chrono::{DateTime, Utc};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::protocol::tags::ApubTag;
use lemmy_db_schema::source::{activity::ActivitySendTargets, person::Person};
use lemmy_utils::error::LemmyResult;
use url::Url;

pub mod comment;
pub(crate) mod note_wrapper;
pub mod post;
pub mod private_message;

/// Combine an object's ap_id with published/updated timestamp so Create/Update
/// activity IDs stay stable across outbox fetches but change when the object is edited.
pub(crate) fn activity_object_id(
  published_at: DateTime<Utc>,
  updated_at: Option<DateTime<Utc>>,
  ap_id: Url,
) -> Url {
  let timestamp = updated_at.unwrap_or(published_at);
  let mut object_id = ap_id;
  object_id.set_fragment(Some(&timestamp.to_rfc3339()));
  object_id
}

/// From Activitypub `tag` field extract the mentions, and return the inboxes for these users.
/// Used when sending out activity to ensure the mentioned users see it.
async fn tagged_user_inboxes(
  tagged_users: &[ApubTag],
  context: &Data<LemmyContext>,
) -> LemmyResult<ActivitySendTargets> {
  let tagged_users: Vec<_> = tagged_users.iter().flat_map(ApubTag::mention_id).collect();
  let mut inboxes = ActivitySendTargets::empty();
  for t in tagged_users {
    let person = t.dereference(context).await?;
    inboxes.add_inbox(person.shared_inbox_or_inbox());
  }
  Ok(inboxes)
}

/// Extracts the users who are mentioned in a received, federated post.
async fn parse_apub_mentions(
  tags: &[ApubTag],
  context: &Data<LemmyContext>,
) -> LemmyResult<Vec<Person>> {
  let mentions: Vec<_> = tags.iter().filter_map(ApubTag::mention_id).collect();
  let mut res = vec![];
  for m in mentions {
    let Some(person) = m.dereference(context).await?.left() else {
      continue;
    };
    if person.local {
      res.push(person.0);
    }
  }
  Ok(res)
}
