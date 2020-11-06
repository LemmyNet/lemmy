use crate::{
  activities::receive::verify_activity_domains_valid,
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_user,
  inbox::{get_activity_id, is_activity_already_known},
  insert_activity,
  ActorType,
};
use activitystreams::{
  activity::{ActorAndObject, Follow, Undo},
  base::AnyBase,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::{anyhow, Context};
use lemmy_db::{
  community::{Community, CommunityFollower, CommunityFollowerForm},
  user::User_,
  Followable,
};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Allowed activities for community inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Follow,
  Undo,
}

pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming receive to community inboxes.
pub async fn community_inbox(
  request: HttpRequest,
  input: web::Json<AcceptedActivities>,
  path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();

  let path = path.into_inner();
  let community = blocking(&context.pool(), move |conn| {
    Community::read_from_name(&conn, &path)
  })
  .await??;

  let to = activity
    .to()
    .context(location_info!())?
    .to_owned()
    .single_xsd_any_uri();
  if Some(community.actor_id()?) != to {
    return Err(anyhow!("Activity delivered to wrong community").into());
  }

  info!(
    "Community {} received activity {:?}",
    &community.name, &activity
  );
  let user_uri = activity
    .actor()?
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  info!(
    "Community {} inbox received activity {:?} from {}",
    community.name,
    &activity.id_unchecked(),
    &user_uri
  );
  check_is_apub_id_valid(user_uri)?;

  let request_counter = &mut 0;
  let user = get_or_fetch_and_upsert_user(&user_uri, &context, request_counter).await?;

  verify_signature(&request, &user)?;

  let activity_id = get_activity_id(&activity, user_uri)?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let res = match kind {
    ValidTypes::Follow => handle_follow(any_base, user, community, &context).await,
    ValidTypes::Undo => handle_undo_follow(any_base, user, community, &context).await,
  };

  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;
  res
}

/// Handle a follow request from a remote user, adding the user as follower and returning an
/// Accept activity.
async fn handle_follow(
  activity: AnyBase,
  user: User_,
  community: Community,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let follow = Follow::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&follow, user.actor_id()?, false)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower, but ignore the error.
  blocking(&context.pool(), move |conn| {
    CommunityFollower::follow(&conn, &community_follower_form).ok()
  })
  .await?;

  community.send_accept_follow(follow, context).await?;

  Ok(HttpResponse::Ok().finish())
}

/// Handle `Undo/Follow` from a user, removing the user from followers list.
async fn handle_undo_follow(
  activity: AnyBase,
  user: User_,
  community: Community,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, user.actor_id()?, true)?;

  let object = undo.object().to_owned().one().context(location_info!())?;
  let follow = Follow::from_any_base(object)?.context(location_info!())?;
  verify_activity_domains_valid(&follow, user.actor_id()?, false)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they aren't a follower, but ignore the error.
  blocking(&context.pool(), move |conn| {
    CommunityFollower::unfollow(&conn, &community_follower_form).ok()
  })
  .await?;

  Ok(HttpResponse::Ok().finish())
}
