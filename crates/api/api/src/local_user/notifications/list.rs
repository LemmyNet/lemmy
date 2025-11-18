use crate::hide_modlog_names;
use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::{
  ListNotifications,
  ListNotificationsResponse,
  NotificationView,
  impls::NotificationQuery,
};
use lemmy_diesel_utils::pagination::PaginationCursorBuilder;
use lemmy_utils::error::LemmyResult;

pub async fn list_notifications(
  Query(data): Query<ListNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListNotificationsResponse>> {
  let hide_modlog_names = hide_modlog_names(Some(&local_user_view), None, &context).await;
  let notifications = NotificationQuery {
    type_: data.type_,
    unread_only: data.unread_only,
    show_bot_accounts: Some(local_user_view.local_user.show_bot_accounts),
    page_cursor: data.page_cursor,
    hide_modlog_names: Some(hide_modlog_names),
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view.person)
  .await?;

  Ok(Json(ListNotificationsResponse {
    notifications: notifications.data,
    next_page: notifications.next_page,
    prev_page: notifications.prev_page,
  }))
}
