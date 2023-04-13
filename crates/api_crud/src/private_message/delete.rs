use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{DeletePrivateMessage, PrivateMessageResponse},
  utils::get_local_user_view_from_jwt,
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::private_message::{PrivateMessage, PrivateMessageUpdateForm},
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeletePrivateMessage {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(self, context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &DeletePrivateMessage = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = PrivateMessage::read(context.pool(), private_message_id).await?;
    if local_user_view.person.id != orig_private_message.creator_id {
      return Err(LemmyError::from_message("no_private_message_edit_allowed"));
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let deleted = data.deleted;
    PrivateMessage::update(
      context.pool(),
      private_message_id,
      &PrivateMessageUpdateForm::builder()
        .deleted(Some(deleted))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_private_message"))?;

    context
      .send_pm_ws_message(
        &UserOperationCrud::DeletePrivateMessage,
        data.private_message_id,
        websocket_id,
      )
      .await
  }
}
