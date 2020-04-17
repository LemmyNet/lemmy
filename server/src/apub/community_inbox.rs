use crate::apub::activities::accept_follow;
use crate::apub::fetcher::fetch_remote_user;
use crate::db::community::{Community, CommunityFollower, CommunityFollowerForm};
use crate::db::Followable;
use activitystreams::activity::Follow;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use log::debug;
use serde::Deserialize;
use url::Url;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
}

/// Handler for all incoming activities to community inboxes.
pub async fn community_inbox(
  input: web::Json<CommunityAcceptedObjects>,
  path: web::Path<String>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  debug!(
    "Community {} received activity {:?}",
    &path.into_inner(),
    &input
  );
  match input {
    CommunityAcceptedObjects::Follow(f) => handle_follow(&f, conn),
  }
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
fn handle_follow(follow: &Follow, conn: &PgConnection) -> Result<HttpResponse, Error> {
  // TODO: make sure this is a local community
  let community_uri = follow
    .follow_props
    .get_object_xsd_any_uri()
    .unwrap()
    .to_string();
  let community = Community::read_from_actor_id(conn, &community_uri)?;
  let user_uri = follow
    .follow_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let user = fetch_remote_user(&Url::parse(&user_uri)?, conn)?;
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };
  CommunityFollower::follow(&conn, &community_follower_form)?;
  accept_follow(&follow)?;
  Ok(HttpResponse::Ok().finish())
}
