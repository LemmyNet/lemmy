use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{PasswordReset, PasswordResetResponse},
  utils::{blocking, send_password_reset_email},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for PasswordReset {
  type Response = PasswordResetResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PasswordResetResponse, LemmyError> {
    let data: &PasswordReset = self;

    // Fetch that email
    let email = data.email.clone();
    let local_user_view = blocking(context.pool(), move |conn| {
      LocalUserView::find_by_email(conn, &email)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_that_username_or_email"))?;

    // Email the pure token to the user.
    send_password_reset_email(&local_user_view, context.pool(), context.settings()).await?;
    Ok(PasswordResetResponse {})
  }
}
