use super::*;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
  Undo(Undo),
}

// TODO Consolidate community and user inboxes into a single shared one
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
    &community_name, &input
  );
  match input {
    CommunityAcceptedObjects::Follow(f) => {
      handle_follow(&f, &request, &community_name, db, chat_server)
    }
    CommunityAcceptedObjects::Undo(u) => {
      handle_undo_follow(&u, &request, &community_name, db, chat_server)
    }
  }
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
fn handle_follow(
  follow: &Follow,
  request: &HttpRequest,
  community_name: &str,
  db: DbPoolParam,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let user_uri = follow
    .follow_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let _community_uri = follow
    .follow_props
    .get_object_xsd_any_uri()
    .unwrap()
    .to_string();

  let conn = db.get()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  let community = Community::read_from_name(&conn, &community_name)?;

  verify(&request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&follow)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

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
  request: &HttpRequest,
  community_name: &str,
  db: DbPoolParam,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let follow = undo
    .undo_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .into_concrete::<Follow>()?;

  let user_uri = follow
    .follow_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();

  let _community_uri = follow
    .follow_props
    .get_object_xsd_any_uri()
    .unwrap()
    .to_string();

  let conn = db.get()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  let community = Community::read_from_name(&conn, &community_name)?;

  verify(&request, &user.public_key.unwrap())?;

  // Insert the received activity into the activity table
  let activity_form = activity::ActivityForm {
    user_id: user.id,
    data: serde_json::to_value(&follow)?,
    local: false,
    updated: None,
  };
  activity::Activity::create(&conn, &activity_form)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  CommunityFollower::ignore(&conn, &community_follower_form).ok();

  Ok(HttpResponse::Ok().finish())
}
