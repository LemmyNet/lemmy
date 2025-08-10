use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::{context::LemmyContext, utils::is_mod_or_admin};
use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_views_community_follower::{
  api::GetCommunityPendingFollowsCountResponse,
  CommunityFollowerView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn get_pending_follows_count(
  community_id: Path<CommunityId>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetCommunityPendingFollowsCountResponse>> {
  let community_id = community_id.into_inner();
  is_mod_or_admin(&mut context.pool(), &local_user_view, community_id).await?;
  let count =
    CommunityFollowerView::count_approval_required(&mut context.pool(), community_id).await?;
  Ok(Json(GetCommunityPendingFollowsCountResponse { count }))
}
