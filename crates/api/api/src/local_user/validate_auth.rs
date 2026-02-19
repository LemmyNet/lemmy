use actix_web::{
  HttpRequest,
  web::{Data, Json},
};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{local_user_view_from_jwt, read_auth_token},
};
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

/// Returns an error message if the auth token is invalid for any reason. Necessary because other
/// endpoints silently treat any call with invalid auth as unauthenticated.
pub async fn validate_auth(
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let jwt = read_auth_token(&req)?;
  if let Some(jwt) = jwt {
    local_user_view_from_jwt(&jwt, &context).await?;
  } else {
    return Err(LemmyErrorType::NotLoggedIn.into());
  }
  Ok(Json(SuccessResponse::default()))
}
