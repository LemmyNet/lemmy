use activitypub_federation::{config::Data, traits::Actor};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::protocol::tags::ApubTag;
use lemmy_db_schema::source::{activity::ActivitySendTargets, person::Person};
use lemmy_utils::error::LemmyResult;

pub mod comment;
pub(crate) mod note_wrapper;
pub mod post;
pub mod private_message;

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
    let person = m.dereference(context).await?.0;
    if person.local {
      res.push(person);
    }
  }
  Ok(res)
}
