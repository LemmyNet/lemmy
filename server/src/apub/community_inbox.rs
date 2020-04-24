use super::*;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
}

/// Handler for all incoming activities to community inboxes.
pub async fn community_inbox(
  request: HttpRequest,
  input: web::Json<CommunityAcceptedObjects>,
  path: web::Path<String>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let community_name = path.into_inner();
  debug!(
    "Community {} received activity {:?}",
    &community_name,
    &input
  );
  match input {
    CommunityAcceptedObjects::Follow(f) => handle_follow(&f, &request, &community_name, db, chat_server),
  }
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
fn handle_follow(
  follow: &Follow,
  request: &HttpRequest,
  community_name: &str,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
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
  let community = Community::read_from_name(&conn, &community_name)?;

  verify(&request, &user.public_key.unwrap())?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower
  CommunityFollower::follow(&conn, &community_follower_form)?;

  accept_follow(&follow, &conn)?;
  Ok(HttpResponse::Ok().finish())
}
