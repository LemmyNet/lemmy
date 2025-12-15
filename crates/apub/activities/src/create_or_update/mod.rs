use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::Actor};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::person::ApubPerson,
  protocol::page::ApubTag,
  utils::mentions::Mention,
};
use lemmy_db_schema::source::{activity::ActivitySendTargets, person::Person};
use lemmy_utils::error::LemmyResult;

pub mod comment;
pub(crate) mod note_wrapper;
pub mod post;
pub mod private_message;

async fn tagged_user_inboxes(
  tagged_users: Vec<ApubTag>,
  context: &Data<LemmyContext>,
) -> LemmyResult<ActivitySendTargets> {
  let tagged_users: Vec<ObjectId<ApubPerson>> = tagged_users
    .into_iter()
    .flat_map(|u| {
      if let ApubTag::Mention(m) = u {
        Some(m)
      } else {
        None
      }
    })
    .map(|t| t.href.clone())
    .map(ObjectId::from)
    .collect();
  let mut inboxes = ActivitySendTargets::empty();
  for t in tagged_users {
    let person = t.dereference(&context).await?;
    inboxes.add_inbox(person.shared_inbox_or_inbox());
  }
  Ok(inboxes)
}

async fn parse_apub_mentions(
  tags: Vec<ApubTag>,
  context: &Data<LemmyContext>,
) -> LemmyResult<Vec<Person>> {
  let mentions: Vec<Mention> = tags
    .iter()
    .filter_map(|t| {
      if let ApubTag::Mention(m) = t {
        Some(m)
      } else {
        None
      }
    })
    .collect();
  // TODO: resolve, filter local
  let mut res = vec![];
  for m in mentions {
    m.dereference(context).await?;
  }
  Ok(res)
}
