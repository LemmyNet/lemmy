use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  newtypes::PrivateMessageId,
  source::private_message::{PrivateMessage, PrivateMessageUpdateForm},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_private_message::{
  api::{DeletePrivateMessage, PrivateMessageResponse},
  PrivateMessageView,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn delete_private_message(
  private_message_id: Path<PrivateMessageId>,
  data: Json<DeletePrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  // Checking permissions
  let private_message_id = private_message_id.into_inner();
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id).await?;
  if local_user_view.person.id != orig_private_message.creator_id {
    Err(LemmyErrorType::EditPrivateMessageNotAllowed)?
  }

  // Doing the update
  let deleted = data.deleted;
  let private_message = PrivateMessage::update(
    &mut context.pool(),
    private_message_id,
    &PrivateMessageUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeletePrivateMessage(local_user_view.person, private_message, data.deleted),
    &context,
  )?;

  let view = PrivateMessageView::read(&mut context.pool(), private_message_id).await?;
  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
