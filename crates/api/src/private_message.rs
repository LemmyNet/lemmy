use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  person::{MarkPrivateMessageAsRead, PrivateMessageResponse},
};
use lemmy_db_schema::{source::private_message::PrivateMessage, traits::Crud};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for MarkPrivateMessageAsRead {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(self, context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &MarkPrivateMessageAsRead = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.recipient_id {
      return Err(LemmyError::from_message(
        "couldnt_update_private_message".into(),
      ));
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let read = data.read;
    blocking(context.pool(), move |conn| {
      PrivateMessage::update_read(conn, private_message_id, read)
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_update_private_message".into()))?;

    // No need to send an apub update
    let op = UserOperation::MarkPrivateMessageAsRead;
    send_pm_ws_message(data.private_message_id, op, websocket_id, context).await
  }
}
