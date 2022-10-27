use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  private_message::{EditPrivateMessage, PrivateMessageResponse},
  utils::{blocking, get_local_user_view_from_jwt, local_site_to_slur_regex},
};
use lemmy_apub::protocol::activities::{
  create_or_update::private_message::CreateOrUpdatePrivateMessage,
  CreateOrUpdateType,
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    private_message::{PrivateMessage, PrivateMessageUpdateForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_utils::{error::LemmyError, utils::remove_slurs, ConnectionId};
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditPrivateMessage {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(self, context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &EditPrivateMessage = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    let local_site = blocking(context.pool(), LocalSite::read).await??;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.creator_id {
      return Err(LemmyError::from_message("no_private_message_edit_allowed"));
    }

    // Doing the update
    let content_slurs_removed = remove_slurs(&data.content, &local_site_to_slur_regex(&local_site));
    let private_message_id = data.private_message_id;
    let updated_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::update(
        conn,
        private_message_id,
        &PrivateMessageUpdateForm::builder()
          .content(Some(content_slurs_removed))
          .updated(Some(Some(naive_now())))
          .build(),
      )
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_private_message"))?;

    // Send the apub update
    CreateOrUpdatePrivateMessage::send(
      updated_private_message.into(),
      &local_user_view.person.into(),
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    let op = UserOperationCrud::EditPrivateMessage;
    send_pm_ws_message(data.private_message_id, op, websocket_id, context).await
  }
}
