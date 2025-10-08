use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_community_mod_of_any_or_admin_action};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_community_follower_approval::{
  api::{ListCommunityPendingFollows, ListCommunityPendingFollowsResponse},
  PendingFollowerView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn get_pending_follows_list(
  data: Query<ListCommunityPendingFollows>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListCommunityPendingFollowsResponse>> {
  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;
  let all_communities =
    data.all_communities.unwrap_or_default() && local_user_view.local_user.admin;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PendingFollowerView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let items = PendingFollowerView::list_approval_required(
    &mut context.pool(),
    local_user_view.person.id,
    all_communities,
    data.unread_only.unwrap_or_default(),
    cursor_data,
    data.page_back,
    data.limit,
  )
  .await?;

  let next_page = items.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = items.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListCommunityPendingFollowsResponse {
    items,
    next_page,
    prev_page,
  }))
}
