use crate::db::post::{Post, PostForm};
use crate::db::Crud;
use activitystreams::activity::{Accept, Create, Update};
use activitystreams::object::Page;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use log::debug;
use serde::Deserialize;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum UserAcceptedObjects {
  Create(Create),
  Update(Update),
  Accept(Accept),
}

#[derive(Deserialize)]
pub struct Params {
  user_name: String,
}

/// Handler for all incoming activities to user inboxes.
pub async fn user_inbox(
  input: web::Json<UserAcceptedObjects>,
  params: web::Query<Params>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  debug!("User {} received activity: {:?}", &params.user_name, &input);

  match input {
    UserAcceptedObjects::Create(c) => handle_create(&c, conn),
    UserAcceptedObjects::Update(u) => handle_update(&u, conn),
    UserAcceptedObjects::Accept(a) => handle_accept(&a, conn),
  }
}

/// Handle create activities and insert them in the database.
fn handle_create(create: &Create, conn: &PgConnection) -> Result<HttpResponse, Error> {
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
fn handle_update(update: &Update, conn: &PgConnection) -> Result<HttpResponse, Error> {
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
fn handle_accept(_accept: &Accept, _conn: &PgConnection) -> Result<HttpResponse, Error> {
  // TODO: make sure that we actually requested a follow
  // TODO: at this point, indicate to the user that they are following the community
  Ok(HttpResponse::Ok().finish())
}
