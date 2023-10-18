use crate::{local_user_view_from_jwt, read_auth_token};
use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_utils::error::{LemmyError, LemmyErrorType};

/// Returns an error message if the auth token is invalid for any reason. Necessary because other
/// endpoints silently treat any call with invalid auth as unauthenticated.
#[tracing::instrument(skip(context))]
pub async fn validate_auth(
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> Result<Json<SuccessResponse>, LemmyError> {
  let jwt = read_auth_token(&req)?;
  if let Some(jwt) = jwt {
    local_user_view_from_jwt(&jwt, &context).await?;
  } else {
    Err(LemmyErrorType::NotLoggedIn)?;
  }
  Ok(Json(SuccessResponse::default()))
}
