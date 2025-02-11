use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{CreatePrivateMessage, PrivateMessageResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_private_messages_enabled,
    get_url_blocklist,
    local_site_to_slur_regex,
    process_markdown,
    send_email_to_user,
  },
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    person_block::PersonBlock,
    private_message::{PrivateMessage, PrivateMessageInsertForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{markdown::markdown_to_html, validation::is_valid_body_field},
};

pub async fn create_private_message(
  data: Json<CreatePrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  PersonBlock::read(
    &mut context.pool(),
    data.recipient_id,
    local_user_view.person.id,
  )
  .await?;

  check_private_messages_enabled(&local_user_view)?;

  // Don't allow local sends to people who have private messages disabled
  let recipient_local_user_opt = LocalUserView::read_person(&mut context.pool(), data.recipient_id)
    .await
    .ok();
  if let Some(recipient_local_user) = recipient_local_user_opt {
    check_private_messages_enabled(&recipient_local_user)?;
  }

  let private_message_form = PrivateMessageInsertForm::new(
    local_user_view.person.id,
    data.recipient_id,
    content.clone(),
  );

  let inserted_private_message = PrivateMessage::create(&mut context.pool(), &private_message_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreatePrivateMessage)?;

  let view = PrivateMessageView::read(&mut context.pool(), inserted_private_message.id).await?;

  // Send email to the local recipient, if one exists
  if view.recipient.local {
    let recipient_id = data.recipient_id;
    let local_recipient = LocalUserView::read_person(&mut context.pool(), recipient_id).await?;
    let lang = &local_recipient.local_user.interface_i18n_language();
    let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());
    let sender_name = &local_user_view.person.name;
    let content = markdown_to_html(&content);
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
  )?;

  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
