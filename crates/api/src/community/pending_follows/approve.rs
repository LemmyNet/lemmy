use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::ApproveCommunityPendingFollower,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_mod_or_admin,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::community::{CommunityFollower, CommunityFollowerForm},
  traits::Followable,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn post_pending_follows_approve(
  data: Json<ApproveCommunityPendingFollower>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view.person,
    data.community_id,
  )
  .await?;

  let activity_data = if data.approve {
    CommunityFollower::approve(
      &mut context.pool(),
      data.community_id,
      data.follower_id,
      local_user_view.person.id,
    )
    .await?;
    SendActivityData::AcceptFollower(data.community_id, data.follower_id)
  } else {
    let form = CommunityFollowerForm::new(data.community_id, data.follower_id);
    CommunityFollower::unfollow(&mut context.pool(), &form).await?;
    SendActivityData::RejectFollower(data.community_id, data.follower_id)
  };
  ActivityChannel::submit_activity(activity_data, &context)?;

  Ok(Json(SuccessResponse::default()))
}
