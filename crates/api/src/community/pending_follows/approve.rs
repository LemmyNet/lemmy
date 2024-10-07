use actix_web::web::{Data, Json};
use lemmy_api_common::{
  community::CommunityPendingFollowsApprove,
  context::LemmyContext,
  utils::is_mod_or_admin,
  SuccessResponse,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn post_pending_follows_approve(
  data: Json<CommunityPendingFollowsApprove>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view.person,
    data.community_id,
  )
  .await?;
  todo!();
  Ok(Json(SuccessResponse::default()))
}
