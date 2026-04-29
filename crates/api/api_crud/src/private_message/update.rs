use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  context::LemmyContext,
  notify::notify_private_message,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_local_user_valid,
    check_private_messages_enabled,
    get_url_blocklist,
    process_markdown,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::{
    person::PersonActions,
    private_message::{PrivateMessage, PrivateMessageUpdateForm},
  },
  traits::Blockable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_private_message::{
  PrivateMessageView,
  api::{EditPrivateMessage, PrivateMessageResponse},
};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::validation::is_valid_body_field,
};

pub async fn edit_private_message(
  Json(data): Json<EditPrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  check_local_user_valid(&local_user_view)?;
  // Checking permissions
  let private_message_id = data.private_message_id;
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id).await?;
  if local_user_view.person.id != orig_private_message.creator_id {
    return Err(LemmyErrorType::EditPrivateMessageNotAllowed.into());
  }

  // Doing the update
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let content = process_markdown(
    &data.content,
    &slur_regex,
    &url_blocklist,
    &local_site,
    &context,
  )
  .await?;
  is_valid_body_field(&content, false)?;

  PersonActions::read_block(
    &mut context.pool(),
    orig_private_message.recipient_id,
    local_user_view.person.id,
  )
  .await?;

  check_private_messages_enabled(&local_user_view)?;

  let private_message_id = data.private_message_id;
  let mut form = PrivateMessageUpdateForm {
    content: Some(content),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  form = plugin_hook_before("local_private_message_before_update", form).await?;
  let private_message =
    PrivateMessage::update(&mut context.pool(), private_message_id, &form).await?;
  plugin_hook_after("local_private_message_after_update", &private_message);

  let view = PrivateMessageView::read(
    &mut context.pool(),
    private_message_id,
    Some(&local_user_view.person),
  )
  .await?;

  notify_private_message(&view, false, &context);

  ActivityChannel::submit_activity(
    SendActivityData::UpdatePrivateMessage(view.clone()),
    &context,
  )?;

  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
