use lemmy_api_common::{blocking, person::PrivateMessageResponse};
use lemmy_db_schema::PrivateMessageId;
use lemmy_db_views::{local_user_view::LocalUserView, private_message_view::PrivateMessageView};
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperationCrud};

pub mod create_or_update;
pub mod delete;
pub mod undo_delete;

async fn send_websocket_message(
  private_message_id: PrivateMessageId,
  op: UserOperationCrud,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let message = blocking(context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message_id)
  })
  .await??;
  let res = PrivateMessageResponse {
    private_message_view: message,
  };

  // Send notifications to the local recipient, if one exists
  let recipient_id = res.private_message_view.recipient.id;
  let local_recipient_id = blocking(context.pool(), move |conn| {
    LocalUserView::read_person(conn, recipient_id)
  })
  .await??
  .local_user
  .id;

  context.chat_server().do_send(SendUserRoomMessage {
    op,
    response: res,
    local_recipient_id,
    websocket_id: None,
  });

  Ok(())
}
