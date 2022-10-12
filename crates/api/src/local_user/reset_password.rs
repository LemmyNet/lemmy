use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{PasswordReset, PasswordResetResponse},
  utils::{blocking, local_site_to_email_config, send_password_reset_email},
};
use lemmy_db_schema::source::local_site::LocalSite;
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

    let local_site = blocking(context.pool(), LocalSite::read).await??;

    // Fetch that email
    let email = data.email.to_lowercase();
    let local_user_view = blocking(context.pool(), move |conn| {
      LocalUserView::find_by_email(conn, &email)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_that_username_or_email"))?;

    // Email the pure token to the user.
    let email_config = local_site_to_email_config(&local_site)?;
    send_password_reset_email(
      &local_user_view,
      context.pool(),
      context.settings(),
      &email_config,
    )
    .await?;
    Ok(PasswordResetResponse {})
  }
}
