use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  private_message::{CreatePrivateMessage, PrivateMessageResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_private_messages_enabled, get_url_blocklist, process_markdown, slur_regex},
};
use lemmy_db_schema::{
  source::{
    person::PersonActions,
    private_message::{PrivateMessage, PrivateMessageInsertForm},
  },
  traits::{Blockable, Crud},
};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_email::notifications::send_private_message_email;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::validation::is_valid_body_field,
};

pub async fn create_private_message(
  data: Json<CreatePrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  PersonActions::read_block(
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

  let mut form = PrivateMessageInsertForm::new(
    local_user_view.person.id,
    data.recipient_id,
    content.clone(),
  );

  form = plugin_hook_before("before_create_local_private_message", form).await?;
  let inserted_private_message = PrivateMessage::create(&mut context.pool(), &form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreatePrivateMessage)?;
  plugin_hook_after(
    "after_create_local_private_message",
    &inserted_private_message,
  )?;

  let view = PrivateMessageView::read(&mut context.pool(), inserted_private_message.id).await?;

  // Send email to the local recipient, if one exists
  if view.recipient.local {
    let local_recipient =
      LocalUserView::read_person(&mut context.pool(), data.recipient_id).await?;
    send_private_message_email(
      &local_user_view,
      &local_recipient,
      &content,
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
