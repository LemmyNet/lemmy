use activitypub_federation::config::Data;
use actix_web::{cookie::Cookie, HttpRequest, HttpResponse};
use lemmy_api_common::{context::LemmyContext, utils::AUTH_COOKIE_NAME};
use lemmy_db_schema::source::login_token::LoginToken;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn logout(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  // TODO: need to retrieve auth token. middleware could write it to request extensions
  let jwt = todo!();
  LoginToken::invalidate(&mut context.pool(), jwt).await?;

  let mut res = HttpResponse::Ok().finish();
  let mut cookie = Cookie::new(AUTH_COOKIE_NAME, "");
  res.add_removal_cookie(&cookie)?;
  Ok(res)
}
