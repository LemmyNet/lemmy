use super::{generate_activity_id, send_lemmy_activity};
use crate::protocol::following::{
  accept::AcceptFollow,
  follow::Follow,
  reject::RejectFollow,
  undo_follow::UndoFollow,
};
use activitypub_federation::{config::Data, kinds::activity::FollowType, traits::Activity};
use either::Either::*;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{
  CommunityOrMulti,
  UserOrCommunityOrMulti,
  community::ApubCommunity,
  person::ApubPerson,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{
    activity::{ActivitySendTargets, SentActivityForm},
    community::Community,
    person::Person,
  },
};
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde::Serialize;

pub(crate) mod accept;
pub(crate) mod follow;
pub(crate) mod reject;
pub(crate) mod undo_follow;

pub fn send_follow(
  target: CommunityOrMulti,
  person: Person,
  follow: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<SentActivityForm> {
  let actor: ApubPerson = person.into();
  if follow {
    Follow::send(&actor, &target, context)
  } else {
    UndoFollow::send(&actor, &target, context)
  }
}

pub fn send_accept_or_reject_follow(
  community: ApubCommunity,
  person: ApubPerson,
  accepted: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<SentActivityForm> {
  let follow = Follow {
    actor: person.ap_id.clone().into(),
    to: Some([community.ap_id.clone().into()]),
    object: community.ap_id.clone().into(),
    kind: FollowType::Follow,
    id: generate_activity_id(FollowType::Follow, context)?,
  };
  if accepted {
    AcceptFollow::send(follow, Right(Left(community)), Left(person), context)
  } else {
    RejectFollow::send(follow, community, person, context)
  }
}

/// Wrapper type which is needed because we cant implement ActorT for Either.
fn send_activity_from_user_or_community_or_multi<A>(
  activity: A,
  target: UserOrCommunityOrMulti,
  send_targets: ActivitySendTargets,
) -> LemmyResult<SentActivityForm>
where
  A: Activity + Serialize + Send + Sync + Clone + Activity<Error = LemmyError>,
{
  match target {
    Left(user) => send_lemmy_activity(activity, &user, send_targets, true),
    Right(Left(community)) => send_lemmy_activity(activity, &community, send_targets, true),
    Right(Right(multi)) => send_lemmy_activity(activity, &multi, send_targets, true),
  }
}
