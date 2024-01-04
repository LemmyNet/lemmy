use crate::read_auth_token;
use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_common::{claims::Claims, context::LemmyContext, SuccessResponse};
use lemmy_utils::error::{LemmyError, LemmyErrorExt2, LemmyErrorType};

/// Returns an error message if the auth token is invalid for any reason. Necessary because other
/// endpoints silently treat any call with invalid auth as unauthenticated.
#[tracing::instrument(skip(context))]
pub async fn validate_auth(
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> Result<Json<SuccessResponse>, LemmyError> {
  let jwt = read_auth_token(&req)?;
  if let Some(jwt) = jwt {
    Claims::validate(&jwt, &context)
      .await
      .with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
  } else {
    Err(LemmyErrorType::NotLoggedIn)?;
  }
  Ok(Json(SuccessResponse::default()))
}
