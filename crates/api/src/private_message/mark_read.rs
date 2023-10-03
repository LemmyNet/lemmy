use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{MarkPrivateMessageAsRead, PrivateMessageResponse},
};
use lemmy_db_schema::{
  source::private_message::{PrivateMessage, PrivateMessageUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn mark_pm_as_read(
  data: Json<MarkPrivateMessageAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PrivateMessageResponse>, LemmyError> {
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
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdatePrivateMessage)?;

  let view = PrivateMessageView::read(&mut context.pool(), private_message_id).await?;
  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
