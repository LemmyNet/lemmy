use crate::{http::create_apub_response, ActorType, ToApub};
use activitystreams::{
  base::BaseExt,
  collection::{CollectionExt, OrderedCollection},
};
use actix_web::{body::Body, web, HttpResponse};
use lemmy_db::user::User_;
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
  let user = blocking(context.pool(), move |conn| {
    User_::find_by_email_or_username(conn, &user_name)
  })
  .await??;
  let u = user.to_apub(context.pool()).await?;
  Ok(create_apub_response(&u))
}

pub async fn get_apub_user_outbox(
  info: web::Path<UserQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let user = blocking(context.pool(), move |conn| {
    User_::read_from_name(&conn, &info.user_name)
  })
  .await??;
  let mut collection = OrderedCollection::new();
  collection
    .set_many_items(Vec::<Url>::new())
    .set_context(activitystreams::context())
    .set_id(user.get_outbox_url()?)
    .set_total_items(0_u64);
  Ok(create_apub_response(&collection))
}
