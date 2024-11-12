use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  community::{GetCommunityPendingFollowsCount, GetCommunityPendingFollowsCountResponse},
  context::LemmyContext,
  utils::is_mod_or_admin,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::LemmyResult;

pub async fn get_pending_follows_count(
  data: Query<GetCommunityPendingFollowsCount>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetCommunityPendingFollowsCountResponse>> {
  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view.person,
    data.community_id,
  )
  .await?;
  let count =
    CommunityFollowerView::count_approval_required(&mut context.pool(), data.community_id).await?;
  Ok(Json(GetCommunityPendingFollowsCountResponse { count }))
}
