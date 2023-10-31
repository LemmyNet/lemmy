use crate::read_auth_token;
use activitypub_federation::config::Data;
use actix_web::{cookie::Cookie, HttpRequest, HttpResponse};
use lemmy_api_common::{context::LemmyContext, utils::AUTH_COOKIE_NAME, SuccessResponse};
use lemmy_db_schema::source::login_token::LoginToken;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn logout(
  req: HttpRequest,
  // require login
  _local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let jwt = read_auth_token(&req)?.ok_or(LemmyErrorType::NotLoggedIn)?;
  LoginToken::invalidate(&mut context.pool(), &jwt).await?;

  let mut res = HttpResponse::Ok().json(SuccessResponse::default());
  let cookie = Cookie::new(AUTH_COOKIE_NAME, "");
  res.add_removal_cookie(&cookie)?;
  Ok(res)
}
