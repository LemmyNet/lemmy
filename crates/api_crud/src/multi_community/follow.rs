use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{community::FollowMultiCommunity, context::LemmyContext};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityFollow, MultiCommunityFollowForm},
  traits::Crud,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn follow_multi_community(
  data: Json<FollowMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<MultiCommunityFollow>> {
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
  let res = if data.follow {
    MultiCommunity::follow(&mut context.pool(), &form).await?
  } else {
    MultiCommunity::unfollow(&mut context.pool(), multi_community_id, person_id).await?
  };

  // TODO: federate if multi-comm is remote

  Ok(Json(res))
}
