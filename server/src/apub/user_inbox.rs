use super::*;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum UserAcceptedObjects {
  Create(Create),
  Update(Update),
  Accept(Accept),
}

/// Handler for all incoming activities to user inboxes.
pub async fn user_inbox(
  request: HttpRequest,
  input: web::Json<UserAcceptedObjects>,
  path: web::Path<String>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  let username = path.into_inner();
  debug!(
    "User {} received activity: {:?}",
    &username,
    &input
  );

  match input {
    UserAcceptedObjects::Create(c) => handle_create(&c, &request, &username, &conn),
    UserAcceptedObjects::Update(u) => handle_update(&u, &request, &username, &conn),
    UserAcceptedObjects::Accept(a) => handle_accept(&a, &request, &username, &conn),
  }
}

/// Handle create activities and insert them in the database.
fn handle_create(
  create: &Create,
  request: &HttpRequest,
  username: &str,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  // TODO before this even gets named, because we don't know what type of object it is, we need
  // to parse this out
  let community_uri = create
    .create_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  // TODO: should do this in a generic way so we dont need to know if its a user or a community
  let user = fetch_remote_user(&Url::parse(&community_uri)?, conn)?;
  verify(request, &user.public_key.unwrap())?;

  let page = create
    .create_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, conn)?;
  Post::create(conn, &post)?;
  // TODO: send the new post out via websocket
  Ok(HttpResponse::Ok().finish())
}

/// Handle update activities and insert them in the database.
fn handle_update(
  update: &Update,
  request: &HttpRequest,
  username: &str,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  let community_uri = update
    .update_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let user = fetch_remote_user(&Url::parse(&community_uri)?, conn)?;
  verify(request, &user.public_key.unwrap())?;

  let page = update
    .update_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, conn)?;
  let id = Post::read_from_apub_id(conn, &post.ap_id)?.id;
  Post::update(conn, id, &post)?;
  // TODO: send the new post out via websocket
  Ok(HttpResponse::Ok().finish())
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
