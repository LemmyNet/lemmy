use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{newtypes::NotificationId, source::notification::Notification};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::api::MarkNotificationAsRead;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn mark_notification_as_read(
  notification_id: Path<NotificationId>,
  // TODO: A boolean read field is also passed, but is not used.
  // What to do with this?
  _data: Json<MarkNotificationAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  Notification::mark_read_by_id_and_person(
    &mut context.pool(),
    notification_id.into_inner(),
    local_user_view.person.id,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
