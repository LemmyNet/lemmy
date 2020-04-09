use crate::db::post::{Post, PostForm};
use crate::db::Crud;
use activitystreams::activity::Create;
use activitystreams::object::Page;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;

// TODO: need a proper actor that has this inbox

pub async fn create_inbox(
  create: web::Json<Create>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let page = create
    .create_props
    .get_object_base_box()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, &db.get().unwrap())?;
  Post::create(&db.get().unwrap(), &post)?;
  // TODO: send the new post out via websocket
  dbg!(&post);
  Ok(HttpResponse::Ok().finish())
}
