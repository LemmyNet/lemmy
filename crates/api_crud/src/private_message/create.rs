use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_person_block,
  get_local_user_view_from_jwt,
  person::{CreatePrivateMessage, PrivateMessageResponse},
};
use lemmy_apub::{
  generate_local_apub_endpoint,
  protocol::activities::{
    private_message::create_or_update::CreateOrUpdatePrivateMessage,
    CreateOrUpdateType,
  },
  EndpointType,
};
use lemmy_db_schema::{
  source::private_message::{PrivateMessage, PrivateMessageForm},
  traits::Crud,
};
use lemmy_db_views::local_user_view::LocalUserView;
use lemmy_utils::{utils::remove_slurs, ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{
  send::{send_email_to_user, send_pm_ws_message},
  LemmyContext,
  UserOperationCrud,
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreatePrivateMessage {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &CreatePrivateMessage = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let content_slurs_removed =
      remove_slurs(&data.content.to_owned(), &context.settings().slur_regex());

    check_person_block(local_user_view.person.id, data.recipient_id, context.pool()).await?;

    let private_message_form = PrivateMessageForm {
      content: content_slurs_removed.to_owned(),
      creator_id: local_user_view.person.id,
      recipient_id: data.recipient_id,
      ..PrivateMessageForm::default()
    };

    let inserted_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::create(conn, &private_message_form)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(e) => {
        return Err(ApiError::err("couldnt_create_private_message", e).into());
      }
    };

    let inserted_private_message_id = inserted_private_message.id;
    let protocol_and_hostname = context.settings().get_protocol_and_hostname();
    let updated_private_message = blocking(
      context.pool(),
      move |conn| -> Result<PrivateMessage, LemmyError> {
        let apub_id = generate_local_apub_endpoint(
          EndpointType::PrivateMessage,
          &inserted_private_message_id.to_string(),
          &protocol_and_hostname,
        )?;
        Ok(PrivateMessage::update_ap_id(
          conn,
          inserted_private_message_id,
          apub_id,
        )?)
      },
    )
    .await?
    .map_err(|e| ApiError::err("couldnt_create_private_message", e))?;

    CreateOrUpdatePrivateMessage::send(
      updated_private_message.into(),
      &local_user_view.person.into(),
      CreateOrUpdateType::Create,
      context,
    )
    .await?;

    let res = send_pm_ws_message(
      inserted_private_message.id,
      UserOperationCrud::CreatePrivateMessage,
      websocket_id,
      context,
    )
    .await?;

    // Send email to the local recipient, if one exists
    if res.private_message_view.recipient.local {
      let recipient_id = data.recipient_id;
      let local_recipient = blocking(context.pool(), move |conn| {
        LocalUserView::read_person(conn, recipient_id)
      })
      .await??;
      send_email_to_user(
        &local_recipient,
        "Private Message from",
        "Private Message",
        &content_slurs_removed,
        &context.settings(),
      );
    }

    Ok(res)
  }
}
