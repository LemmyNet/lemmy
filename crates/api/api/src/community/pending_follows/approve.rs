use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_mod_or_admin,
};
use lemmy_db_schema::source::community::CommunityActions;
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_community::api::ApproveCommunityPendingFollower;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn post_pending_follows_approve(
  data: Json<ApproveCommunityPendingFollower>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  is_mod_or_admin(&mut context.pool(), &local_user_view, data.community_id).await?;

  let (state, activity_data) = if data.approve {
    (
      CommunityFollowerState::Accepted,
      SendActivityData::AcceptFollower(data.community_id, data.follower_id),
    )
  } else {
    (
      CommunityFollowerState::Denied,
      SendActivityData::RejectFollower(data.community_id, data.follower_id),
    )
  };
  CommunityActions::approve_private_community_follower(
    &mut context.pool(),
    data.community_id,
    data.follower_id,
    local_user_view.person.id,
    state,
  )
  .await?;
  ActivityChannel::submit_activity(activity_data, &context)?;

  Ok(Json(SuccessResponse::default()))
}
