use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  person::{MarkPrivateMessageAsRead, PrivateMessageResponse},
};
use lemmy_db_queries::{source::private_message::PrivateMessage_, Crud};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_db_views::{local_user_view::LocalUserView, private_message_view::PrivateMessageView};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for MarkPrivateMessageAsRead {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &MarkPrivateMessageAsRead = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.recipient_id {
      return Err(ApiError::err("couldnt_update_private_message").into());
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let read = data.read;
    match blocking(context.pool(), move |conn| {
      PrivateMessage::update_read(conn, private_message_id, read)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(ApiError::err("couldnt_update_private_message").into()),
    };

    // No need to send an apub update
    let private_message_id = data.private_message_id;
    let private_message_view = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, private_message_id)
    })
    .await??;

    let res = PrivateMessageResponse {
      private_message_view,
    };

    // Send notifications to the local recipient, if one exists
    let recipient_id = orig_private_message.recipient_id;
    if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await?
    {
      let local_recipient_id = local_recipient.local_user.id;
      context.chat_server().do_send(SendUserRoomMessage {
        op: UserOperation::MarkPrivateMessageAsRead,
        response: res.clone(),
        local_recipient_id,
        websocket_id,
      });
    }

    Ok(res)
  }
}
