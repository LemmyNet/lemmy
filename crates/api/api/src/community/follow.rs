use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_deleted_removed, check_local_user_valid},
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    community::{Community, CommunityActions, CommunityFollowerForm},
  },
  traits::{Crud, Followable},
};
use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
use lemmy_db_views_community::{
  api::{CommunityResponse, FollowCommunity},
  CommunityView,
};
use lemmy_db_views_community_person_ban::CommunityPersonBanView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn follow_community(
  data: Json<FollowCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  check_local_user_valid(&local_user_view)?;
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  let person_id = local_user_view.person.id;

  if data.follow {
    // Only run these checks for local community, in case of remote community the local
    // state may be outdated. Can't use check_community_user_action() here as it only allows
    // actions from existing followers for private community (so following would be impossible).
    if community.local {
      check_community_deleted_removed(&community)?;
      CommunityPersonBanView::check(&mut context.pool(), person_id, community.id).await?;
    }

    let follow_state = if community.visibility == CommunityVisibility::Private {
      // Private communities require manual approval
      CommunityFollowerState::ApprovalRequired
    } else if community.local {
      // Local follow is accepted immediately
      CommunityFollowerState::Accepted
    } else {
      // remote follow needs to be federated first
      CommunityFollowerState::Pending
    };
    let form = CommunityFollowerForm::new(community.id, person_id, follow_state);

    // Write to db
    CommunityActions::follow(&mut context.pool(), &form).await?;
  } else {
    CommunityActions::unfollow(&mut context.pool(), person_id, community.id).await?;
  }

  // Send the federated follow
  if !community.local {
    ActivityChannel::submit_activity(
      SendActivityData::FollowCommunity(community, local_user_view.person.clone(), data.follow),
      &context,
    )?;
  }

  let community_id = data.community_id;
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(&local_user_view.local_user),
    false,
  )
  .await?;

  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}
