use crate::db::post::{Post, PostForm};
use crate::db::Crud;
use activitystreams::object::Page;
use activitystreams::{
  object::{Object, ObjectBox},
  primitives::XsdAnyUri,
  Base, BaseBox, PropRefs,
};
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use std::collections::HashMap;

// TODO: need a proper actor that has this inbox

pub async fn inbox(
  input: web::Json<AcceptedObjects>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  match input.kind {
    ValidTypes::Create => handle_create(&input, conn),
    ValidTypes::Update => handle_update(&input, conn),
  }
}

fn handle_create(create: &AcceptedObjects, conn: &PgConnection) -> Result<HttpResponse, Error> {
  let page = create.object.to_owned().to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, conn)?;
  Post::create(conn, &post)?;
  // TODO: send the new post out via websocket
  Ok(HttpResponse::Ok().finish())
}

fn handle_update(update: &AcceptedObjects, conn: &PgConnection) -> Result<HttpResponse, Error> {
  let page = update.object.to_owned().to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, conn)?;
  let id = Post::read_from_apub_id(conn, &post.ap_id)?.id;
  Post::update(conn, id, &post)?;
  // TODO: send the new post out via websocket
  Ok(HttpResponse::Ok().finish())
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedObjects {
  pub id: XsdAnyUri,

  #[serde(rename = "type")]
  pub kind: ValidTypes,

  pub actor: XsdAnyUri,

  pub object: BaseBox,

  #[serde(flatten)]
  ext: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Create,
  Update,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum ValidObjects {
  Id(XsdAnyUri),
  Object(AnyExistingObject),
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize, PropRefs)]
#[serde(rename_all = "camelCase")]
#[prop_refs(Object)]
pub struct AnyExistingObject {
  pub id: XsdAnyUri,

  #[serde(rename = "type")]
  pub kind: String,

  #[serde(flatten)]
  ext: HashMap<String, serde_json::Value>,
}
