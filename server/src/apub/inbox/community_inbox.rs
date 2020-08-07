use crate::{
  apub::{
    check_is_apub_id_valid,
    extensions::signatures::verify,
    fetcher::get_or_fetch_and_upsert_user,
    insert_activity,
    ActorType,
  },
  blocking,
  routes::{ChatServerParam, DbPoolParam},
  LemmyError,
};
use activitystreams::{
  activity::{ActorAndObject, Follow, Undo},
  base::AnyBase,
  prelude::*,
};
use actix_web::{client::Client, web, HttpRequest, HttpResponse};
use anyhow::anyhow;
use lemmy_db::{
  community::{Community, CommunityFollower, CommunityFollowerForm},
  user::User_,
  Followable,
};
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
  db: DbPoolParam,
  client: web::Data<Client>,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();

  let path = path.into_inner();
  let community = blocking(&db, move |conn| Community::read_from_name(&conn, &path)).await??;

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
  let user_uri = activity.actor()?.as_single_xsd_any_uri().unwrap();
  check_is_apub_id_valid(user_uri)?;

  let user = get_or_fetch_and_upsert_user(&user_uri, &client, &db).await?;

  verify(&request, &user)?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().unwrap();
  let user_id = user.id;
  let res = match kind {
    ValidTypes::Follow => handle_follow(any_base, user, community, &client, &db).await,
    ValidTypes::Undo => handle_undo_follow(any_base, user, community, &db).await,
  };

  insert_activity(user_id, activity.clone(), false, &db).await?;
  res
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
async fn handle_follow(
  activity: AnyBase,
  user: User_,
  community: Community,
  client: &Client,
  db: &DbPoolParam,
) -> Result<HttpResponse, LemmyError> {
  let follow = Follow::from_any_base(activity)?.unwrap();
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower, but ignore the error.
  blocking(db, move |conn| {
    CommunityFollower::follow(&conn, &community_follower_form).ok()
  })
  .await?;

  community.send_accept_follow(follow, &client, db).await?;

  Ok(HttpResponse::Ok().finish())
}

async fn handle_undo_follow(
  activity: AnyBase,
  user: User_,
  community: Community,
  db: &DbPoolParam,
) -> Result<HttpResponse, LemmyError> {
  let _undo = Undo::from_any_base(activity)?.unwrap();

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they aren't a follower, but ignore the error.
  blocking(db, move |conn| {
    CommunityFollower::unfollow(&conn, &community_follower_form).ok()
  })
  .await?;

  Ok(HttpResponse::Ok().finish())
}
