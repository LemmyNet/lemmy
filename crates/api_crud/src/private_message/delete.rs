use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{DeletePrivateMessage, PrivateMessageResponse},
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::private_message::{PrivateMessage, PrivateMessageUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn delete_private_message(
  data: Json<DeletePrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  // Checking permissions
  let private_message_id = data.private_message_id;
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPrivateMessage)?;
  if local_user_view.person.id != orig_private_message.creator_id {
    Err(LemmyErrorType::EditPrivateMessageNotAllowed)?
  }

  // Doing the update
  let private_message_id = data.private_message_id;
  let deleted = data.deleted;
  let private_message = PrivateMessage::update(
    &mut context.pool(),
    private_message_id,
    &PrivateMessageUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdatePrivateMessage)?;

  ActivityChannel::submit_activity(
    SendActivityData::DeletePrivateMessage(local_user_view.person, private_message, data.deleted),
    &context,
  )
  .await?;

  let view = PrivateMessageView::read(&mut context.pool(), private_message_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPrivateMessage)?;
  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
