use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{CreatePrivateMessage, PrivateMessageResponse},
  utils::{
    check_person_block,
    generate_local_apub_endpoint,
    get_interface_language,
    get_local_user_view_from_jwt,
    local_site_to_slur_regex,
    send_email_to_user,
    EndpointType,
  },
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    private_message::{PrivateMessage, PrivateMessageInsertForm, PrivateMessageUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyError, utils::slurs::remove_slurs, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreatePrivateMessage {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(self, context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &CreatePrivateMessage = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let content_slurs_removed = remove_slurs(
      &data.content.clone(),
      &local_site_to_slur_regex(&local_site),
    );

    check_person_block(local_user_view.person.id, data.recipient_id, context.pool()).await?;

    let private_message_form = PrivateMessageInsertForm::builder()
      .content(content_slurs_removed.clone())
      .creator_id(local_user_view.person.id)
      .recipient_id(data.recipient_id)
      .build();

    let inserted_private_message =
      match PrivateMessage::create(context.pool(), &private_message_form).await {
        Ok(private_message) => private_message,
        Err(e) => {
          return Err(LemmyError::from_error_message(
            e,
            "couldnt_create_private_message",
          ));
        }
      };

    let inserted_private_message_id = inserted_private_message.id;
    let protocol_and_hostname = context.settings().get_protocol_and_hostname();
    let apub_id = generate_local_apub_endpoint(
      EndpointType::PrivateMessage,
      &inserted_private_message_id.to_string(),
      &protocol_and_hostname,
    )?;
    PrivateMessage::update(
      context.pool(),
      inserted_private_message.id,
      &PrivateMessageUpdateForm::builder()
        .ap_id(Some(apub_id))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_private_message"))?;

    let res = context
      .send_pm_ws_message(
        &UserOperationCrud::CreatePrivateMessage,
        inserted_private_message.id,
        websocket_id,
      )
      .await?;

    // Send email to the local recipient, if one exists
    if res.private_message_view.recipient.local {
      let recipient_id = data.recipient_id;
      let local_recipient = LocalUserView::read_person(context.pool(), recipient_id).await?;
      let lang = get_interface_language(&local_recipient);
      let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());
      send_email_to_user(
        &local_recipient,
        &lang.notification_private_message_subject(&local_recipient.person.name),
        &lang.notification_private_message_body(
          inbox_link,
          &content_slurs_removed,
          &local_recipient.person.name,
        ),
        context.settings(),
      );
    }

    Ok(res)
  }
}
