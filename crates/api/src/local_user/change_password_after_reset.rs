use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{LoginResponse, PasswordChangeAfterReset},
  utils::{blocking, password_length_check},
};
use lemmy_db_schema::source::{
  local_user::LocalUser,
  password_reset_request::PasswordResetRequest,
};
use lemmy_utils::{claims::Claims, error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for PasswordChangeAfterReset {
  type Response = LoginResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &PasswordChangeAfterReset = self;

    // Fetch the user_id from the token
    let token = data.token.clone();
    let local_user_id = blocking(context.pool(), move |conn| {
      PasswordResetRequest::read_from_token(conn, &token).map(|p| p.local_user_id)
    })
    .await??;

    password_length_check(&data.password)?;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(LemmyError::from_message("passwords_dont_match"));
    }

    // Update the user with the new password
    let password = data.password.clone();
    let updated_local_user = blocking(context.pool(), move |conn| {
      LocalUser::update_password(conn, local_user_id, &password)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_user"))?;

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
