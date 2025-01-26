use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListInbox, ListInboxResponse},
};
use lemmy_db_views::{combined::inbox_combined_view::InboxCombinedQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn list_inbox(
  data: Query<ListInbox>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListInboxResponse>> {
  let unread_only = data.unread_only;
  let type_ = data.type_;
  let person_id = local_user_view.person.id;
  let show_bot_accounts = Some(local_user_view.local_user.show_bot_accounts);

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let inbox = InboxCombinedQuery {
    type_,
    unread_only,
    show_bot_accounts,
    page_after,
    page_back,
  }
  .list(&mut context.pool(), person_id)
  .await?;

  Ok(Json(ListInboxResponse { inbox }))
}
