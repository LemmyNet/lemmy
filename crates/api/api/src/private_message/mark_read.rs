use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::private_message::{PrivateMessage, PrivateMessageUpdateForm},
  traits::Crud,
};
use lemmy_db_views_api_misc::SuccessResponse;
use lemmy_db_views_inbox_combined::api::MarkPrivateMessageAsRead;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn mark_pm_as_read(
  data: Json<MarkPrivateMessageAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Checking permissions
  let private_message_id = data.private_message_id;
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id).await?;
  if local_user_view.person.id != orig_private_message.recipient_id {
    Err(LemmyErrorType::CouldntUpdatePrivateMessage)?
  }

  // Doing the update
  let private_message_id = data.private_message_id;
  let read = data.read;
  PrivateMessage::update(
    &mut context.pool(),
    private_message_id,
    &PrivateMessageUpdateForm {
      read: Some(read),
      ..Default::default()
    },
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
