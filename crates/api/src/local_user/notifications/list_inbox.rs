use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListInbox, ListInboxResponse},
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views::{
  combined::inbox_combined_view::InboxCombinedQuery,
  structs::{InboxCombinedView, LocalUserView},
};
use lemmy_utils::error::LemmyResult;

pub async fn list_inbox(
  data: Query<ListInbox>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListInboxResponse>> {
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(InboxCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let inbox = InboxCombinedQuery {
    type_: data.type_,
    unread_only: data.unread_only,
    show_bot_accounts: Some(local_user_view.local_user.show_bot_accounts),
    cursor_data,
    page_back: data.page_back,
  }
  .list(&mut context.pool(), person_id, local_instance_id)
  .await?;

  let next_page = inbox.last().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListInboxResponse { inbox, next_page }))
}
