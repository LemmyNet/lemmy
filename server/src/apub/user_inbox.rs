use super::*;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum UserAcceptedObjects {
  Accept(Accept),
}

/// Handler for all incoming activities to user inboxes.
pub async fn user_inbox(
  request: HttpRequest,
  input: web::Json<UserAcceptedObjects>,
  path: web::Path<String>,
  db: DbPoolParam,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  let username = path.into_inner();
  debug!("User {} received activity: {:?}", &username, &input);

  match input {
    UserAcceptedObjects::Accept(a) => handle_accept(&a, &request, &username, &conn),
  }
}

/// Handle accepted follows.
fn handle_accept(
  accept: &Accept,
  request: &HttpRequest,
  username: &str,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  let community_uri = accept
    .accept_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let community = get_or_fetch_and_upsert_remote_community(&community_uri, conn)?;
  verify(request, &community.public_key.unwrap())?;

  let user = User_::read_from_name(&conn, username)?;

  // Now you need to add this to the community follower
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower
  CommunityFollower::follow(&conn, &community_follower_form)?;

  // TODO: make sure that we actually requested a follow
  // TODO: at this point, indicate to the user that they are following the community
  Ok(HttpResponse::Ok().finish())
}
