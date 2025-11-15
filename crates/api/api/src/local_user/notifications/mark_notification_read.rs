use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::notification::Notification;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::api::MarkNotificationAsRead;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn mark_notification_as_read(
  data: Json<MarkNotificationAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  Notification::mark_read_by_id_and_person(
    &mut context.pool(),
    data.notification_id,
    local_user_view.person.id,
    data.read,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
