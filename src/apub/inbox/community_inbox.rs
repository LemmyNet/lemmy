use crate::{
  apub::{
    check_is_apub_id_valid,
    extensions::signatures::verify,
    fetcher::get_or_fetch_and_upsert_user,
    insert_activity,
    ActorType,
  },
  LemmyContext,
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
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Follow,
  Undo,
}

pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming activities to community inboxes.
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

  if !community.local {
    return Err(
      anyhow!(
        "Received activity is addressed to remote community {}",
        &community.actor_id
      )
      .into(),
    );
  }
  debug!(
    "Community {} received activity {:?}",
    &community.name, &activity
  );
  let user_uri = activity
    .actor()?
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  check_is_apub_id_valid(user_uri)?;

  let user = get_or_fetch_and_upsert_user(&user_uri, &context).await?;

  verify(&request, &user)?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let user_id = user.id;
  let res = match kind {
    ValidTypes::Follow => handle_follow(any_base, user, community, &context).await,
    ValidTypes::Undo => handle_undo_follow(any_base, user, community, &context).await,
  };

  insert_activity(user_id, activity.clone(), false, context.pool()).await?;
  res
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
async fn handle_follow(
  activity: AnyBase,
  user: User_,
  community: Community,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let follow = Follow::from_any_base(activity)?.context(location_info!())?;
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

async fn handle_undo_follow(
  activity: AnyBase,
  user: User_,
  community: Community,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let _undo = Undo::from_any_base(activity)?.context(location_info!())?;

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
