use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{EditPrivateMessage, PrivateMessageResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{local_site_to_slur_regex, local_user_view_from_jwt, sanitize_html},
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    private_message::{PrivateMessage, PrivateMessageUpdateForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{slurs::remove_slurs, validation::is_valid_body_field},
};

#[tracing::instrument(skip(context))]
pub async fn update_private_message(
  data: Json<EditPrivateMessage>,
  context: Data<LemmyContext>,
) -> Result<Json<PrivateMessageResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
  let local_site = LocalSite::read(&mut context.pool()).await?;

  // Checking permissions
  let private_message_id = data.private_message_id;
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id).await?;
  if local_user_view.person.id != orig_private_message.creator_id {
    Err(LemmyErrorType::EditPrivateMessageNotAllowed)?
  }

  // Doing the update
  let content = sanitize_html(&data.content);
  let content = remove_slurs(&content, &local_site_to_slur_regex(&local_site));
  is_valid_body_field(&Some(content.clone()), false)?;

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

  let view = PrivateMessageView::read(&mut context.pool(), private_message_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdatePrivateMessage(view.clone()),
    &context,
  )
  .await?;

  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
