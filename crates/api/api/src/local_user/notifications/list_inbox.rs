use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::{
  impls::NotificationQuery,
  ListInbox,
  ListInboxResponse,
  NotificationView,
};
use lemmy_utils::error::LemmyResult;

pub async fn list_inbox(
  data: Query<ListInbox>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListInboxResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(NotificationView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let inbox = NotificationQuery {
    type_: data.type_,
    unread_only: data.unread_only,
    show_bot_accounts: Some(local_user_view.local_user.show_bot_accounts),
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = inbox.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = inbox.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListInboxResponse {
    inbox,
    next_page,
    prev_page,
  }))
}
