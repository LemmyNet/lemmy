use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  community::{ListCommunityPendingFollows, ListCommunityPendingFollowsResponse},
  context::LemmyContext,
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_views::structs::{CommunityFollowerView, LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn get_pending_follows_list(
  data: Query<ListCommunityPendingFollows>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListCommunityPendingFollowsResponse>> {
  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;
  let all_communities =
    data.all_communities.unwrap_or_default() && local_user_view.local_user.admin;
  let items = CommunityFollowerView::list_approval_required(
    &mut context.pool(),
    local_user_view.person.id,
    all_communities,
    data.pending_only.unwrap_or_default(),
    data.page,
    data.limit,
  )
  .await?;
  Ok(Json(ListCommunityPendingFollowsResponse { items }))
}
