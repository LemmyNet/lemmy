use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::{CommunityResponse, FollowCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_deleted_removed, check_user_valid},
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    community::{Community, CommunityFollower, CommunityFollowerForm, CommunityFollowerState},
  },
  traits::{Crud, Followable},
  CommunityVisibility,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::{CommunityPersonBanView, CommunityView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn follow_community(
  data: Json<FollowCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  let mut form = CommunityFollowerForm::new(community.id, local_user_view.person.id);

  if data.follow {
    // Same checks as check_community_user_action(), except don't check for private community
    check_user_valid(&local_user_view.person)?;
    check_community_deleted_removed(&community).await?;
    CommunityPersonBanView::check(&mut context.pool(), local_user_view.person.id, community.id)
      .await?;

    // Local follow is accepted immediately, remote follow needs to be federated first
    form.state = if community.local {
      Some(CommunityFollowerState::Accepted)
    } else {
      Some(CommunityFollowerState::Pending)
    };

    // Private communities require manual approval
    if community.visibility == CommunityVisibility::Private {
      form.state = Some(CommunityFollowerState::ApprovalRequired);
    }

    // Write to db
    CommunityFollower::follow(&mut context.pool(), &form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
  } else {
    CommunityFollower::unfollow(&mut context.pool(), &form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
  }

  // Send the federated follow
  if !community.local {
    ActivityChannel::submit_activity(
      SendActivityData::FollowCommunity(community, local_user_view.person.clone(), data.follow),
      &context,
    )
    .await?;
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
