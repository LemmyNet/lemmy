use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_local_user_valid,
};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityFollowForm},
  traits::Crud,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_community::api::FollowMultiCommunity;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn follow_multi_community(
  data: Json<FollowMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  check_local_user_valid(&local_user_view)?;
  let multi_community_id = data.multi_community_id;
  let person_id = local_user_view.person.id;
  let multi = MultiCommunity::read(&mut context.pool(), multi_community_id).await?;

  let follow_state = if multi.local {
    CommunityFollowerState::Accepted
  } else {
    CommunityFollowerState::Pending
  };
  let form = MultiCommunityFollowForm {
    multi_community_id,
    person_id,
    follow_state,
  };

  if data.follow {
    MultiCommunity::follow(&mut context.pool(), &form).await?;
  } else {
    MultiCommunity::unfollow(&mut context.pool(), person_id, multi_community_id).await?;
  }

  if !multi.local {
    ActivityChannel::submit_activity(
      SendActivityData::FollowMultiCommunity(multi, local_user_view.person.clone(), data.follow),
      &context,
    )?;
  }

  Ok(Json(SuccessResponse::default()))
}
