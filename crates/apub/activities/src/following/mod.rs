use super::{generate_activity_id, send_lemmy_activity};
use crate::{
  activity_lists::AnnouncableActivities,
  community::mod_inboxes,
  protocol::{
    community::announce::AnnounceActivity,
    following::{
      accept::AcceptFollow,
      follow::Follow,
      reject::RejectFollow,
      undo_follow::UndoFollow,
    },
  },
};
use activitypub_federation::{
  config::Data,
  kinds::activity::FollowType,
  traits::{Activity, Object},
};
use either::Either::{self, *};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{
  CommunityOrMulti,
  UserOrCommunityOrMulti,
  community::ApubCommunity,
  person::ApubPerson,
};
use lemmy_db_schema::source::{
  activity::ActivitySendTargets,
  community::Community,
  person::Person,
};
use lemmy_db_schema_file::{PersonId, newtypes::CommunityId};
use lemmy_diesel_utils::{dburl::DbUrl, traits::Crud};
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde::Serialize;

pub(crate) mod accept;
pub(crate) mod follow;
pub(crate) mod reject;
pub(crate) mod undo_follow;

pub async fn send_follow(
  target: CommunityOrMulti,
  person: Person,
  follow: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let actor: ApubPerson = person.into();

  // If the community is local, notify remote mods of the follow request or undo
  if let Left(community) = &target
    && community.local
  {
    let follow_or_undo = if follow {
      Left(Follow::new(&actor, &target, context)?)
    } else {
      Right(UndoFollow::new(&actor, &target, context)?)
    };
    return notify_remote_mods(follow_or_undo, context, community).await;
  }

  if follow {
    Follow::send(&actor, &target, context).await
  } else {
    UndoFollow::send(&actor, &target, context).await
  }
}

pub async fn send_accept_or_reject_follow(
  mod_: Person,
  community_id: CommunityId,
  person_id: PersonId,
  follow_activity_id: Option<DbUrl>,
  accepted: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let community = Community::read(&mut context.pool(), community_id).await?;
  let person = Person::read(&mut context.pool(), person_id).await?;

  // This fallback should never be needed as we always store `community_actions.follow_activity_id`.
  // But it needs to be an option so the type system thinks it can be missing
  let fallback_id = generate_activity_id(FollowType::Follow, context)?;

  let follow = Follow {
    actor: person.ap_id.into(),
    to: Some([community.ap_id.clone().into()]),
    object: community.ap_id.clone().into(),
    kind: FollowType::Follow,
    id: follow_activity_id.map(Into::into).unwrap_or(fallback_id),
  };

  // A remote mod can't sign as the community, so they send the accept or reject to the
  // instance hosting the community, which forwards it to the follower
  if !community.local {
    return send_as_mod(follow, accepted, &mod_.into(), &community.into(), context).await;
  }

  if accepted {
    AcceptFollow::send(follow, context).await
  } else {
    RejectFollow::send(follow, context).await
  }
}

async fn send_as_mod(
  follow: Follow,
  accepted: bool,
  mod_: &ApubPerson,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let inbox = ActivitySendTargets::to_inbox(community.inbox_url.clone().into());
  if accepted {
    let accept = AcceptFollow::new(mod_.id(), follow, context)?;
    send_lemmy_activity(context, accept, mod_, inbox, true).await
  } else {
    let reject = RejectFollow::new(mod_.id(), follow, context)?;
    send_lemmy_activity(context, reject, mod_, inbox, true).await
  }
}

/// Wrapper type which is needed because we cant implement ActorT for Either.
async fn send_activity_from_user_or_community_or_multi<A>(
  context: &Data<LemmyContext>,
  activity: A,
  target: UserOrCommunityOrMulti,
  send_targets: ActivitySendTargets,
) -> LemmyResult<()>
where
  A: Activity + Serialize + Send + Sync + Clone + Activity<Error = LemmyError>,
{
  match target {
    Left(user) => send_lemmy_activity(context, activity, &user, send_targets, true).await,
    Right(Left(community)) => {
      send_lemmy_activity(context, activity, &community, send_targets, true).await
    }
    Right(Right(multi)) => send_lemmy_activity(context, activity, &multi, send_targets, true).await,
  }
}

async fn notify_remote_mods(
  follow_or_undo: Either<Follow, UndoFollow>,
  context: &Data<LemmyContext>,
  community: &ApubCommunity,
) -> LemmyResult<()> {
  let announce = AnnounceActivity::new(
    match follow_or_undo {
      Left(follow) => AnnouncableActivities::Follow(follow).try_into()?,
      Right(undo) => AnnouncableActivities::UndoFollow(undo).try_into()?,
    },
    &community.clone(),
    None,
    context,
  )?;
  let mut inboxes = ActivitySendTargets::empty();
  let mod_inboxes = mod_inboxes(context, community.id).await?;
  inboxes.add_inboxes(mod_inboxes);

  send_lemmy_activity(context, announce, community, inboxes, true).await?;
  Ok(())
}
