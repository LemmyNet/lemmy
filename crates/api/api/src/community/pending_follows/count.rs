use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_community_mod_of_any_or_admin_action};
use lemmy_db_views_community_follower_approval::{
  PendingFollowerView,
  api::GetCommunityPendingFollowsCountResponse,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn get_pending_follows_count(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetCommunityPendingFollowsCountResponse>> {
  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;
  let count =
    PendingFollowerView::count_approval_required(&mut context.pool(), local_user_view.person.id)
      .await?;
  Ok(Json(GetCommunityPendingFollowsCountResponse { count }))
}
