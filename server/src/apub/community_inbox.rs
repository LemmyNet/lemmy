use crate::{
  apub::{
    extensions::signatures::verify,
    fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user},
    ActorType,
  },
  blocking,
  db::{
    activity::insert_activity,
    community::{Community, CommunityFollower, CommunityFollowerForm},
    user::User_,
    Followable,
  },
  routes::{ChatServerParam, DbPoolParam},
  LemmyError,
};
use activitystreams::activity::Undo;
use activitystreams_new::activity::Follow;
use actix_web::{client::Client, web, HttpRequest, HttpResponse};
use log::debug;
use serde::Deserialize;
use std::fmt::Debug;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
  Undo(Undo),
}

impl CommunityAcceptedObjects {
  fn follow(&self) -> Result<Follow, LemmyError> {
    match self {
      CommunityAcceptedObjects::Follow(f) => Ok(f.to_owned()),
      CommunityAcceptedObjects::Undo(u) => Ok(
        u.undo_props
          .get_object_base_box()
          .to_owned()
          .unwrap()
          .to_owned()
          .into_concrete::<Follow>()?,
      ),
    }
  }
}

/// Handler for all incoming activities to community inboxes.
pub async fn community_inbox(
  request: HttpRequest,
  input: web::Json<CommunityAcceptedObjects>,
  path: web::Path<String>,
  db: DbPoolParam,
  client: web::Data<Client>,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let input = input.into_inner();

  let path = path.into_inner();
  let community = blocking(&db, move |conn| Community::read_from_name(&conn, &path)).await??;

  if !community.local {
    return Err(
      format_err!(
        "Received activity is addressed to remote community {}",
        &community.actor_id
      )
      .into(),
    );
  }
  debug!(
    "Community {} received activity {:?}",
    &community.name, &input
  );
  let follow = input.follow()?;
  let user_uri = follow.actor.as_single_xsd_any_uri().unwrap().to_string();
  let community_uri = follow.object.as_single_xsd_any_uri().unwrap().to_string();

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &client, &db).await?;
  let community = get_or_fetch_and_upsert_remote_community(&community_uri, &client, &db).await?;

  verify(&request, &user)?;

  match input {
    CommunityAcceptedObjects::Follow(f) => handle_follow(f, user, community, &client, db).await,
    CommunityAcceptedObjects::Undo(u) => handle_undo_follow(u, user, community, db).await,
  }
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
async fn handle_follow(
  follow: Follow,
  user: User_,
  community: Community,
  client: &Client,
  db: DbPoolParam,
) -> Result<HttpResponse, LemmyError> {
  insert_activity(user.id, follow.clone(), false, &db).await?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower, but ignore the error.
  blocking(&db, move |conn| {
    CommunityFollower::follow(&conn, &community_follower_form).ok()
  })
  .await?;

  community.send_accept_follow(&follow, &client, &db).await?;

  Ok(HttpResponse::Ok().finish())
}

async fn handle_undo_follow(
  undo: Undo,
  user: User_,
  community: Community,
  db: DbPoolParam,
) -> Result<HttpResponse, LemmyError> {
  insert_activity(user.id, undo, false, &db).await?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they aren't a follower, but ignore the error.
  blocking(&db, move |conn| {
    CommunityFollower::unfollow(&conn, &community_follower_form).ok()
  })
  .await?;

  Ok(HttpResponse::Ok().finish())
}
