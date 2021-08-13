use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  person::{DeletePrivateMessage, PrivateMessageResponse},
};
use lemmy_apub::activities::private_message::{
  delete::DeletePrivateMessage as DeletePrivateMessageApub,
  undo_delete::UndoDeletePrivateMessage,
};
use lemmy_db_queries::{source::private_message::PrivateMessage_, Crud, DeleteableOrRemoveable};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeletePrivateMessage {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &DeletePrivateMessage = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.creator_id {
      return Err(ApiError::err("no_private_message_edit_allowed").into());
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let deleted = data.deleted;
    let updated_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, private_message_id, deleted)
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_update_private_message"))?;

    // Send the apub update
    if data.deleted {
      DeletePrivateMessageApub::send(
        &local_user_view.person,
        &updated_private_message.blank_out_deleted_or_removed_info(),
        context,
      )
      .await?;
    } else {
      UndoDeletePrivateMessage::send(&local_user_view.person, &updated_private_message, context)
        .await?;
    }

    let op = UserOperationCrud::DeletePrivateMessage;
    send_pm_ws_message(data.private_message_id, op, websocket_id, context).await
  }
}
