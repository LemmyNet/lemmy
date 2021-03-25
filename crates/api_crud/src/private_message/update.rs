use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  person::{EditPrivateMessage, PrivateMessageResponse},
};
use lemmy_apub::ApubObjectType;
use lemmy_db_queries::{source::private_message::PrivateMessage_, Crud};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_db_views::{local_user_view::LocalUserView, private_message_view::PrivateMessageView};
use lemmy_utils::{utils::remove_slurs, ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPrivateMessage {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &EditPrivateMessage = &self;
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
    let content_slurs_removed = remove_slurs(&data.content);
    let private_message_id = data.private_message_id;
    let updated_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::update_content(conn, private_message_id, &content_slurs_removed)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(ApiError::err("couldnt_update_private_message").into()),
    };

    // Send the apub update
    updated_private_message
      .send_update(&local_user_view.person, context)
      .await?;

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
        op: UserOperationCrud::EditPrivateMessage,
        response: res.clone(),
        local_recipient_id,
        websocket_id,
      });
    }

    Ok(res)
  }
}
