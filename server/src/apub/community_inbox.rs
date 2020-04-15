use crate::apub::activities::accept_follow;
use crate::apub::fetcher::fetch_remote_user;
use crate::db::community::{Community, CommunityFollower, CommunityFollowerForm};
use crate::db::Followable;
use activitystreams::activity::Follow;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use url::Url;

#[serde(untagged)]
#[derive(serde::Deserialize)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
}

pub async fn community_inbox(
  input: web::Json<CommunityAcceptedObjects>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  match input {
    CommunityAcceptedObjects::Follow(f) => handle_follow(&f, conn),
  }
}

fn handle_follow(follow: &Follow, conn: &PgConnection) -> Result<HttpResponse, Error> {
  println!("received follow: {:?}", &follow);

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
  // TODO: insert ID of the user into follows of the community
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };
  CommunityFollower::follow(&conn, &community_follower_form)?;
  accept_follow(&follow)?;
  Ok(HttpResponse::Ok().finish())
}
