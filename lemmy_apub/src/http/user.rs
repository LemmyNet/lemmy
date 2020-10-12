use crate::{http::create_apub_response, ToApub};
use actix_web::{body::Body, web, HttpResponse};
use lemmy_db::user::User_;
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserQuery {
  user_name: String,
}

/// Return the user json over HTTP.
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
