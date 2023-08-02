use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{LoginResponse, PasswordChangeAfterReset},
  utils::password_length_check,
};
use lemmy_db_schema::source::{
  local_user::LocalUser,
  password_reset_request::PasswordResetRequest,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for PasswordChangeAfterReset {
  type Response = LoginResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<LoginResponse, LemmyError> {
    let data: &PasswordChangeAfterReset = self;

    // Fetch the user_id from the token
    let token = data.token.clone();
    let local_user_id = PasswordResetRequest::read_from_token(&mut context.pool(), &token)
      .await
      .map(|p| p.local_user_id)?;

    password_length_check(&data.password)?;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(LemmyErrorType::PasswordsDoNotMatch)?;
    }

    // Update the user with the new password
    let password = data.password.clone();
    LocalUser::update_password(&mut context.pool(), local_user_id, &password)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

    Ok(LoginResponse {
      jwt: None,
      verify_email_sent: false,
      registration_created: false,
    })
  }
}
