use crate::Perform;
use actix_web::web::Data;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{ChangePassword, LoginResponse},
  utils::{local_user_view_from_jwt, password_length_check},
};
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_utils::{
  claims::Claims,
  error::{LemmyError, LemmyErrorType},
};

#[async_trait::async_trait(?Send)]
impl Perform for ChangePassword {
  type Response = LoginResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<LoginResponse, LemmyError> {
    let data: &ChangePassword = self;
    let local_user_view = local_user_view_from_jwt(data.auth.as_ref(), context).await?;

    password_length_check(&data.new_password)?;

    // Make sure passwords match
    if data.new_password != data.new_password_verify {
      return Err(LemmyErrorType::PasswordsDoNotMatch)?;
    }

    // Check the old password
    let valid: bool = verify(
      &data.old_password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(LemmyErrorType::IncorrectLogin)?;
    }

    let local_user_id = local_user_view.local_user.id;
    let new_password = data.new_password.clone();
    let updated_local_user =
      LocalUser::update_password(&mut context.pool(), local_user_id, &new_password).await?;

    // Return the jwt
    Ok(LoginResponse {
      jwt: Some(
        Claims::jwt(
          updated_local_user.id.0,
          &context.secret().jwt_secret,
          &context.settings().hostname,
        )?
        .into(),
      ),
      verify_email_sent: false,
      registration_created: false,
    })
  }
}
