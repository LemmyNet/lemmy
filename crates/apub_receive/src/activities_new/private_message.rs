use activitystreams::activity::kind::CreateType;
use lemmy_api_common::{blocking, person::PrivateMessageResponse};
use lemmy_apub::{objects::FromApub, NoteExt};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_db_views::{local_user_view::LocalUserView, private_message_view::PrivateMessageView};
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperationCrud};
use url::Url;
use lemmy_apub_lib::{ReceiveActivity};
use crate::inbox::new_inbox_routing::Activity;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePrivateMessage {
  actor: Url,
  to: Url,
  object: NoteExt,
  #[serde(rename = "type")]
  kind: CreateType,
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<CreatePrivateMessage> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message = PrivateMessage::from_apub(
      &self.inner.object,
      context,
      self.inner.actor.clone(),
      request_counter,
      false,
    )
    .await?;

    let message = blocking(&context.pool(), move |conn| {
      PrivateMessageView::read(conn, private_message.id)
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
      op: UserOperationCrud::CreatePrivateMessage,
      response: res,
      local_recipient_id,
      websocket_id: None,
    });

    Ok(())
  }
}
