use crate::db::post::{Post, PostForm};
use crate::db::Crud;
use activitystreams::activity::{Accept, Create, Update};
use activitystreams::object::Page;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;

#[serde(untagged)]
#[derive(serde::Deserialize)]
pub enum UserAcceptedObjects {
  Create(Create),
  Update(Update),
  Accept(Accept),
}

pub async fn user_inbox(
  input: web::Json<UserAcceptedObjects>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  match input {
    UserAcceptedObjects::Create(c) => handle_create(&c, conn),
    UserAcceptedObjects::Update(u) => handle_update(&u, conn),
    UserAcceptedObjects::Accept(a) => handle_accept(&a, conn),
  }
}

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

fn handle_accept(accept: &Accept, _conn: &PgConnection) -> Result<HttpResponse, Error> {
  println!("received accept: {:?}", &accept);
  // TODO: at this point, indicate to the user that they are following the community
  Ok(HttpResponse::Ok().finish())
}
