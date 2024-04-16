use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::{CommunityResponse, FollowCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_user_action,
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    community::{Community, CommunityFollower, CommunityFollowerForm},
  },
  traits::{Crud, Followable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn follow_community(
  data: Json<FollowCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommunity)?;
  let mut community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    person_id: local_user_view.person.id,
    pending: false,
  };

  if data.follow {
    if community.local {
      check_community_user_action(&local_user_view.person, community.id, &mut context.pool())
        .await?;

      CommunityFollower::follow(&mut context.pool(), &community_follower_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
    } else {
      // Mark as pending, the actual federation activity is sent via `SendActivity` handler
      community_follower_form.pending = true;
      CommunityFollower::follow(&mut context.pool(), &community_follower_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
    }
  } else {
    CommunityFollower::unfollow(&mut context.pool(), &community_follower_form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
  }

  if !community.local {
    ActivityChannel::submit_activity(
      SendActivityData::FollowCommunity(community, local_user_view.person.clone(), data.follow),
      &context,
    )
    .await?;
  }

  let community_id = data.community_id;
  let person_id = local_user_view.person.id;
  let community_view =
    CommunityView::read(&mut context.pool(), community_id, Some(person_id), false)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?;

  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}
