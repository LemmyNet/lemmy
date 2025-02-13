use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListInbox, ListInboxResponse},
};
use lemmy_db_schema::traits::PageCursorBuilder;
use lemmy_db_views::{combined::inbox_combined_view::InboxCombinedQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn list_inbox(
  data: Query<ListInbox>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListInboxResponse>> {
  let person_id = local_user_view.person.id;

  let inbox = InboxCombinedQuery {
    type_: data.type_,
    unread_only: data.unread_only,
    show_bot_accounts: Some(local_user_view.local_user.show_bot_accounts),
    page_cursor: data.page_cursor.clone(),
    page_back: data.page_back,
  }
  .list(&mut context.pool(), person_id)
  .await?;

  let next_page = inbox.last().map(PageCursorBuilder::cursor);

  Ok(Json(ListInboxResponse { inbox, next_page }))
}
