use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
};
use activitypub_federation::config::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_utils::error::LemmyError;

pub mod accept;
pub mod follow;
pub mod undo_follow;

pub async fn send_follow_community(
  community: Community,
  person: Person,
  follow: bool,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let community: ApubCommunity = community.into();
  let actor: ApubPerson = person.into();
  if follow {
    Follow::send(&actor, &community, context).await
  } else {
    UndoFollow::send(&actor, &community, context).await
  }
}
