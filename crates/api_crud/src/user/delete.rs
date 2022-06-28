use crate::PerformCrud;
use actix_web::web::Data;
use bcrypt::verify;
use lemmy_api_common::{
  person::{DeleteAccount, DeleteAccountResponse},
  utils::{delete_user_account, get_local_user_view_from_jwt},
};
use lemmy_apub::protocol::activities::deletion::delete_user::DeleteUser;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteAccount {
  type Response = DeleteAccountResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(data.auth.as_ref(), context.pool(), context.secret()).await?;

    // Verify the password
    let valid: bool = verify(
      &data.password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(LemmyError::from_message("password_incorrect"));
    }

    delete_user_account(
      local_user_view.person.id,
      context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;
    DeleteUser::send(&local_user_view.person.into(), context).await?;

    Ok(DeleteAccountResponse {})
  }
}
