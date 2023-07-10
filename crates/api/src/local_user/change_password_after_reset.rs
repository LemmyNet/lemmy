use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{LoginResponse, PasswordChangeAfterReset},
  utils::password_length_check,
};
use lemmy_db_schema::{
  source::{local_user::LocalUser, password_reset_request::PasswordResetRequest},
  RegistrationMode,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  claims::Claims,
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
};

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
    let updated_local_user =
      LocalUser::update_password(&mut context.pool(), local_user_id, &password)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

    // Return the jwt if login is allowed
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let jwt = if site_view.local_site.registration_mode == RegistrationMode::RequireApplication
      && !updated_local_user.accepted_application
    {
      None
    } else {
      Some(
        Claims::jwt(
          updated_local_user.id.0,
          &context.secret().jwt_secret,
          &context.settings().hostname,
        )?
        .into(),
      )
    };

    Ok(LoginResponse {
      jwt,
      verify_email_sent: false,
      registration_created: false,
    })
  }
}
