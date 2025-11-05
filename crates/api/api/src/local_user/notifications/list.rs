use crate::hide_modlog_names;
use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::{
  ListNotifications,
  ListNotificationsResponse,
  NotificationView,
  impls::NotificationQuery,
};
use lemmy_utils::error::LemmyResult;

pub async fn list_notifications(
  data: Query<ListNotifications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListNotificationsResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(NotificationView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let hide_modlog_names = hide_modlog_names(Some(&local_user_view), None, &context).await;
  let notifications = NotificationQuery {
    type_: data.type_,
    unread_only: data.unread_only,
    show_bot_accounts: Some(local_user_view.local_user.show_bot_accounts),
    cursor_data,
    page_back: data.page_back,
    hide_modlog_names: Some(hide_modlog_names),
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view.person)
  .await?;

  let next_page = notifications.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = notifications
    .first()
    .map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListNotificationsResponse {
    notifications,
    next_page,
    prev_page,
  }))
}
