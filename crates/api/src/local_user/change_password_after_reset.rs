use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::PasswordChangeAfterReset,
  utils::password_length_check,
  SuccessResponse,
};
use lemmy_db_schema::source::{
  local_user::LocalUser,
  login_token::LoginToken,
  password_reset_request::PasswordResetRequest,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn change_password_after_reset(
  data: Json<PasswordChangeAfterReset>,
  context: Data<LemmyContext>,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Fetch the user_id from the token
  let token = data.token.clone();
  let local_user_id = PasswordResetRequest::read_from_token(&mut context.pool(), &token)
    .await
    .map(|p| p.local_user_id)?;

  password_length_check(&data.password)?;

  // Make sure passwords match
  if data.password != data.password_verify {
    Err(LemmyErrorType::PasswordsDoNotMatch)?
  }

  // Update the user with the new password
  let password = data.password.clone();
  LocalUser::update_password(&mut context.pool(), local_user_id, &password)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

  LoginToken::invalidate_all(&mut context.pool(), local_user_id).await?;

  Ok(Json(SuccessResponse::default()))
}
