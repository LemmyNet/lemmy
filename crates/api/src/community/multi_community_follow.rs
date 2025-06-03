use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::FollowMultiCommunity,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{community_follow_many, community_unfollow_many},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityFollowForm},
  traits::Crud,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_community::impls::CommunityQuery;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn follow_multi_community(
  data: Json<FollowMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let multi_community_id = data.multi_community_id;
  let person_id = local_user_view.person.id;
  let multi = MultiCommunity::read(&mut context.pool(), multi_community_id).await?;
  let site = SiteView::read_local(&mut context.pool()).await?;
  let communities = CommunityQuery {
    local_user: Some(&local_user_view.local_user),
    multi_community_id: Some(multi_community_id),
    ..Default::default()
  }
  .list(&site.site, &mut context.pool())
  .await?;

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

    // Get all communities which are part of the multi and not yet followed by current user
    let to_follow: Vec<_> = communities
      .into_iter()
      .filter(|c| {
        let actions = c.community_actions.clone().unwrap_or_default();
        actions.followed.is_none()
      })
      .map(|c| c.community)
      .collect();

    // Then follow them
    community_follow_many(&local_user_view.person, &to_follow, &context).await?;
  } else {
    MultiCommunity::unfollow(&mut context.pool(), person_id, multi_community_id).await?;

    // Unfollow all communities which were followed as part of multi-comm
    // (is_multi_community_follow=true)
    // TODO: what if a user follows more than one multi-comm
    // containing the same community? then it would get wrongly removed here. so it needs a
    // separate db query to check that.
    let to_unfollow: Vec<_> = communities
      .into_iter()
      .filter(|c| {
        let actions = c.community_actions.clone().unwrap_or_default();
        actions.followed.is_some() && actions.is_multi_community_follow.unwrap_or_default()
      })
      .map(|c| c.community)
      .collect();
    community_unfollow_many(&local_user_view.person, &to_unfollow, &context).await?;
  }

  if !multi.local {
    ActivityChannel::submit_activity(
      SendActivityData::FollowMultiCommunity(multi, local_user_view.person.clone(), data.follow),
      &context,
    )?;
  }

  Ok(Json(SuccessResponse::default()))
}
