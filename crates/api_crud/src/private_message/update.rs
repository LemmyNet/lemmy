use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{EditPrivateMessage, PrivateMessageResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{get_url_blocklist, local_site_to_slur_regex, process_markdown},
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    private_message::{PrivateMessage, PrivateMessageUpdateForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::validation::is_valid_body_field,
};

#[tracing::instrument(skip(context))]
pub async fn update_private_message(
  data: Json<EditPrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  // Checking permissions
  let private_message_id = data.private_message_id;
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPrivateMessage)?;
  if local_user_view.person.id != orig_private_message.creator_id {
    Err(LemmyErrorType::EditPrivateMessageNotAllowed)?
  }

  // Doing the update
  let slur_regex = local_site_to_slur_regex(&local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  let private_message_id = data.private_message_id;
  PrivateMessage::update(
    &mut context.pool(),
    private_message_id,
    &PrivateMessageUpdateForm {
      content: Some(content),
      updated: Some(Some(naive_now())),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdatePrivateMessage)?;

  let view = PrivateMessageView::read(&mut context.pool(), private_message_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPrivateMessage)?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdatePrivateMessage(view.clone()),
    &context,
  )
  .await?;

  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
