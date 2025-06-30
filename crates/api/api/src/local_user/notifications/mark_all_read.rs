use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{
  notification::LocalUserNotification,
  private_message::PrivateMessage,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn mark_all_notifications_read(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let person_id = local_user_view.person.id;

  LocalUserNotification::mark_all_as_read(&mut context.pool(), local_user_view.local_user.id)
    .await?;

  // Mark all private_messages as read
  PrivateMessage::mark_all_as_read(&mut context.pool(), person_id).await?;

  Ok(Json(SuccessResponse::default()))
}
