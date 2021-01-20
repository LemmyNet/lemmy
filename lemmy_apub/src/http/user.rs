use crate::{
  extensions::context::lemmy_context,
  http::{create_apub_response, create_apub_tombstone_response},
  objects::ToApub,
  ActorType,
};
use activitystreams::{
  base::BaseExt,
  collection::{CollectionExt, OrderedCollection},
};
use actix_web::{body::Body, web, HttpResponse};
use lemmy_db_queries::source::user::User;
use lemmy_db_schema::source::user::User_;
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct UserQuery {
  user_name: String,
}

/// Return the ActivityPub json representation of a local user over HTTP.
pub async fn get_apub_user_http(
  info: web::Path<UserQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let user_name = info.into_inner().user_name;
  // TODO: this needs to be able to read deleted users, so that it can send tombstones
  let user = blocking(context.pool(), move |conn| {
    User_::find_by_email_or_username(conn, &user_name)
  })
  .await??;

  if !user.deleted {
    let apub = user.to_apub(context.pool()).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&user.to_tombstone()?))
  }
}

pub async fn get_apub_user_outbox(
  info: web::Path<UserQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let user = blocking(context.pool(), move |conn| {
    User_::read_from_name(&conn, &info.user_name)
  })
  .await??;
  // TODO: populate the user outbox
  let mut collection = OrderedCollection::new();
  collection
    .set_many_items(Vec::<Url>::new())
    .set_many_contexts(lemmy_context()?)
    .set_id(user.get_outbox_url()?)
    .set_total_items(0_u64);
  Ok(create_apub_response(&collection))
}

pub async fn get_apub_user_inbox(
  info: web::Path<UserQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let user = blocking(context.pool(), move |conn| {
    User_::read_from_name(&conn, &info.user_name)
  })
  .await??;

  let mut collection = OrderedCollection::new();
  collection
    .set_id(format!("{}/inbox", user.actor_id).parse()?)
    .set_many_contexts(lemmy_context()?);
  Ok(create_apub_response(&collection))
}
