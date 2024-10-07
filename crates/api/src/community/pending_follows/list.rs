use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  community::{CommunityPendingFollowsListResponse, GetCommunityPendingFollows},
  context::LemmyContext,
  utils::is_mod_or_admin,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::LemmyResult;

pub async fn get_pending_follows_list(
  data: Query<GetCommunityPendingFollows>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityPendingFollowsListResponse>> {
  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view.person,
    data.community_id,
  )
  .await?;
  let items = CommunityFollowerView::list_approval_required(
    &mut context.pool(),
    data.community_id,
    data.pending_only.unwrap_or_default(),
    data.page,
    data.limit,
  )
  .await?;
  Ok(Json(CommunityPendingFollowsListResponse { items }))
}
