use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{VerifyEmail, VerifyEmailResponse},
  utils::{blocking, send_email_verification_success},
};
use lemmy_db_schema::{
  source::{
    email_verification::EmailVerification,
    local_user::{LocalUser, LocalUserForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for VerifyEmail {
  type Response = VerifyEmailResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<usize>,
  ) -> Result<Self::Response, LemmyError> {
    let token = self.token.clone();
    let verification = blocking(context.pool(), move |conn| {
      EmailVerification::read_for_token(conn, &token)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "token_not_found"))?;

    let form = LocalUserForm {
      // necessary in case this is a new signup
      email_verified: Some(true),
      // necessary in case email of an existing user was changed
      email: Some(Some(verification.email)),
      ..LocalUserForm::default()
    };
    let local_user_id = verification.local_user_id;
    blocking(context.pool(), move |conn| {
      LocalUser::update(conn, local_user_id, &form)
    })
    .await??;

    let local_user_view = blocking(context.pool(), move |conn| {
      LocalUserView::read(conn, local_user_id)
    })
    .await??;

    send_email_verification_success(&local_user_view, context.settings())?;

    blocking(context.pool(), move |conn| {
      EmailVerification::delete_old_tokens_for_local_user(conn, local_user_id)
    })
    .await??;

    Ok(VerifyEmailResponse {})
  }
}
