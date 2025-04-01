use super::generate_activity_id;
use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::{
    accept::AcceptFollow,
    follow::Follow,
    reject::RejectFollow,
    undo_follow::UndoFollow,
  },
};
use activitypub_federation::{config::Data, kinds::activity::FollowType};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  source::{community::Community, person::Person},
  traits::Crud,
};
use lemmy_utils::error::LemmyResult;

pub(crate) mod accept;
pub(crate) mod follow;
pub(crate) mod reject;
pub(crate) mod undo_follow;

pub async fn send_follow_community(
  community: Community,
  person: Person,
  follow: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let community: ApubCommunity = community.into();
  let actor: ApubPerson = person.into();
  if follow {
    Follow::send(&actor, &community, context).await
  } else {
    UndoFollow::send(&actor, &community, context).await
  }
}

pub async fn send_accept_or_reject_follow(
  community_id: CommunityId,
  person_id: PersonId,
  accepted: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let community = Community::read(&mut context.pool(), community_id).await?;
  let person = Person::read(&mut context.pool(), person_id).await?;

  let follow = Follow {
    actor: person.ap_id.into(),
    to: Some([community.ap_id.clone().into()]),
    object: community.ap_id.into(),
    kind: FollowType::Follow,
    id: generate_activity_id(
      FollowType::Follow,
      &context.settings().get_protocol_and_hostname(),
    )?,
  };
  if accepted {
    AcceptFollow::send(follow, context).await
  } else {
    RejectFollow::send(follow, context).await
  }
}
