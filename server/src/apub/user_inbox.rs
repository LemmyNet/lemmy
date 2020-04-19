use crate::apub::fetcher::{fetch_remote_community, fetch_remote_user};
use crate::apub::signatures::verify;
use crate::db::post::{Post, PostForm};
use crate::db::Crud;
use activitystreams::activity::{Accept, Create, Update};
use activitystreams::object::Page;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use log::debug;
use serde::Deserialize;
use url::Url;

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
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  // TODO: would be nice if we could do the signature check here, but we cant access the actor property
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  debug!(
    "User {} received activity: {:?}",
    &path.into_inner(),
    &input
  );

  match input {
    UserAcceptedObjects::Create(c) => handle_create(&c, &request, conn),
    UserAcceptedObjects::Update(u) => handle_update(&u, &request, conn),
    UserAcceptedObjects::Accept(a) => handle_accept(&a, &request, conn),
  }
}

/// Handle create activities and insert them in the database.
fn handle_create(
  create: &Create,
  request: &HttpRequest,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
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
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  let community_uri = accept
    .accept_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let community = fetch_remote_community(&Url::parse(&community_uri)?, conn)?;
  verify(request, &community.public_key.unwrap())?;

  // TODO: make sure that we actually requested a follow
  // TODO: at this point, indicate to the user that they are following the community
  Ok(HttpResponse::Ok().finish())
}
