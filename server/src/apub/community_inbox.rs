use crate::{
  apub::{
    extensions::signatures::verify,
    fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user},
    ActorType,
  },
  db::{
    activity::insert_activity,
    community::{Community, CommunityFollower, CommunityFollowerForm},
    user::User_,
    Followable,
  },
  routes::{ChatServerParam, DbPoolParam},
};
use activitystreams::activity::{Follow, Undo};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use diesel::PgConnection;
use failure::{Error, _core::fmt::Debug};
use log::debug;
use serde::Deserialize;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
  Undo(Undo),
}

impl CommunityAcceptedObjects {
  fn follow(&self) -> Result<Follow, Error> {
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

// TODO Consolidate community and user inboxes into a single shared one
/// Handler for all incoming activities to community inboxes.
pub async fn community_inbox(
  request: HttpRequest,
  input: web::Json<CommunityAcceptedObjects>,
  path: web::Path<String>,
  db: DbPoolParam,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let community_name = path.into_inner();
  debug!(
    "Community {} received activity {:?}",
    &community_name, &input
  );
  let follow = input.follow()?;
  let user_uri = follow
    .follow_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let community_uri = follow
    .follow_props
    .get_object_xsd_any_uri()
    .unwrap()
    .to_string();

  let conn = db.get()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  let community = get_or_fetch_and_upsert_remote_community(&community_uri, &conn)?;

  verify(&request, &user)?;

  match input {
    CommunityAcceptedObjects::Follow(f) => handle_follow(&f, &user, &community, &conn),
    CommunityAcceptedObjects::Undo(u) => handle_undo_follow(&u, &user, &community, &conn),
  }
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
fn handle_follow(
  follow: &Follow,
  user: &User_,
  community: &Community,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  insert_activity(&conn, user.id, &follow, false)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower, but ignore the error.
  CommunityFollower::follow(&conn, &community_follower_form).ok();

  community.send_accept_follow(&follow, &conn)?;

  Ok(HttpResponse::Ok().finish())
}

fn handle_undo_follow(
  undo: &Undo,
  user: &User_,
  community: &Community,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  insert_activity(&conn, user.id, &undo, false)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  CommunityFollower::unfollow(&conn, &community_follower_form).ok();

  Ok(HttpResponse::Ok().finish())
}
