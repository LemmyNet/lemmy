use crate::Perform;
use actix_web::web::Data;
use chrono::Duration;
use lemmy_api_common::{
  context::LemmyContext,
  person::{LoginResponse, PasswordChangeAfterReset},
  utils::password_length_check,
};
use lemmy_db_schema::{
  source::{
    local_user::LocalUser,
    password_reset_request::{PasswordResetRequest, PasswordResetRequestForm},
  },
  traits::Crud,
  utils::naive_now,
  RegistrationMode,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{claims::Claims, error::LemmyError};

#[async_trait::async_trait(?Send)]
impl Perform for PasswordChangeAfterReset {
  type Response = LoginResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<LoginResponse, LemmyError> {
    let data: &PasswordChangeAfterReset = self;

    // Fetch the user_id from the token
    let token = data.token.clone();
    let reset_request = PasswordResetRequest::read_unexpired_from_token(context.pool(), &token)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "invalid_password_reset_token"))?;

    password_length_check(&data.password)?;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(LemmyError::from_message("passwords_dont_match"));
    }

    // Expire reset token
    // TODO do this in a transaction along with the user update (blocked by https://github.com/LemmyNet/lemmy/issues/1161)
    PasswordResetRequest::update(
      context.pool(),
      reset_request.id,
      &PasswordResetRequestForm {
        local_user_id: reset_request.local_user_id,
        token_encrypted: reset_request.token_encrypted,
        // Subtract a few seconds in case DB is on separate server and time isn't perfectly synced
        expires_at: naive_now() - Duration::seconds(5),
      },
    )
    .await?;

    // Update the user with the new password
    let password = data.password.clone();
    let updated_local_user =
      LocalUser::update_password(context.pool(), reset_request.local_user_id, &password)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_user"))?;

    // Return the jwt if login is allowed
    let site_view = SiteView::read_local(context.pool()).await?;
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
