use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_mod_or_admin,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::community::CommunityActions,
  traits::Followable,
};
use lemmy_db_views_community::api::ApproveCommunityPendingFollower;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn post_pending_follows_approve(
  community_id: Path<CommunityId>,
  data: Json<ApproveCommunityPendingFollower>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let community_id = community_id.into_inner();
  is_mod_or_admin(&mut context.pool(), &local_user_view, community_id).await?;

  let activity_data = if data.approve {
    CommunityActions::approve_follower(
      &mut context.pool(),
      community_id,
      data.follower_id,
      local_user_view.person.id,
    )
    .await?;
    SendActivityData::AcceptFollower(community_id, data.follower_id)
  } else {
    CommunityActions::unfollow(&mut context.pool(), data.follower_id, community_id).await?;
    SendActivityData::RejectFollower(community_id, data.follower_id)
  };
  ActivityChannel::submit_activity(activity_data, &context)?;

  Ok(Json(SuccessResponse::default()))
}
