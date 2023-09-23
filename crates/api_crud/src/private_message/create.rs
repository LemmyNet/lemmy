use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{CreatePrivateMessage, PrivateMessageResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_person_block, generate_local_apub_endpoint, get_interface_language,
    local_site_to_slur_regex, sanitize_html_api, send_email_to_user, EndpointType,
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
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{slurs::remove_slurs, validation::is_valid_body_field},
};

#[tracing::instrument(skip(context))]
pub async fn create_private_message(
  data: Json<CreatePrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PrivateMessageResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let content = sanitize_html_api(&data.content);
  let content = remove_slurs(&content, &local_site_to_slur_regex(&local_site));
  is_valid_body_field(&Some(content.clone()), false)?;

  check_person_block(
    local_user_view.person.id,
    data.recipient_id,
    &mut context.pool(),
  )
  .await?;

  let private_message_form = PrivateMessageInsertForm::builder()
    .content(content.clone())
    .creator_id(local_user_view.person.id)
    .recipient_id(data.recipient_id)
    .build();

  let inserted_private_message = PrivateMessage::create(&mut context.pool(), &private_message_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreatePrivateMessage)?;

  let inserted_private_message_id = inserted_private_message.id;
  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let apub_id = generate_local_apub_endpoint(
    EndpointType::PrivateMessage,
    &inserted_private_message_id.to_string(),
    &protocol_and_hostname,
  )?;
  PrivateMessage::update(
    &mut context.pool(),
    inserted_private_message.id,
    &PrivateMessageUpdateForm {
      ap_id: Some(apub_id),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntCreatePrivateMessage)?;

  let view = PrivateMessageView::read(&mut context.pool(), inserted_private_message.id).await?;

  // Send email to the local recipient, if one exists
  if view.recipient.local {
    let recipient_id = data.recipient_id;
    let local_recipient = LocalUserView::read_person(&mut context.pool(), recipient_id).await?;
    let lang = get_interface_language(&local_recipient);
    let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());
    let sender_name = &local_user_view.person.name;
    send_email_to_user(
      &local_recipient,
      &lang.notification_private_message_subject(sender_name),
      &lang.notification_private_message_body(inbox_link, &content, sender_name),
      context.settings(),
    )
    .await;
  }

  ActivityChannel::submit_activity(
    SendActivityData::CreatePrivateMessage(view.clone()),
    &context,
  )
  .await?;

  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
