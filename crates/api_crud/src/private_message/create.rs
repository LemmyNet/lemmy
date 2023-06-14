use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{CreatePrivateMessage, PrivateMessageResponse},
  utils::{
    check_person_block,
    generate_local_apub_endpoint,
    get_interface_language,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    send_email_to_user,
    EndpointType,
  },
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    private_message::{PrivateMessage, PrivateMessageInsertForm, PrivateMessageUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_utils::{
  error::LemmyError,
  utils::{slurs::remove_slurs, validation::is_valid_body_field},
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreatePrivateMessage {
  type Response = PrivateMessageResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &CreatePrivateMessage = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let content_slurs_removed = remove_slurs(
      &data.content.clone(),
      &local_site_to_slur_regex(&local_site),
    );
    is_valid_body_field(&Some(content_slurs_removed.clone()))?;

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

    let view = PrivateMessageView::read(context.pool(), inserted_private_message.id).await?;

    // Send email to the local recipient, if one exists
    if view.recipient.local {
      let recipient_id = data.recipient_id;
      let local_recipient = LocalUserView::read_person(context.pool(), recipient_id).await?;
      let lang = get_interface_language(&local_recipient);
      let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());
      let sender_name = &local_user_view.person.name;
      send_email_to_user(
        &local_recipient,
        &lang.notification_private_message_subject(sender_name),
        &lang.notification_private_message_body(inbox_link, &content_slurs_removed, sender_name),
        context.settings(),
      );
    }

    Ok(PrivateMessageResponse {
      private_message_view: view,
    })
  }
}
