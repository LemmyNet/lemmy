use super::{check_multi_community_creator, send_federation_update};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions},
    multi_community::MultiCommunity,
  },
  traits::{Crud, Followable},
};
use lemmy_db_views_community::api::CreateOrDeleteMultiCommunityEntry;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{api::SuccessResponse, SiteView};
use lemmy_utils::error::LemmyResult;

pub async fn delete_multi_community_entry(
  data: Json<CreateOrDeleteMultiCommunityEntry>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let multi = check_multi_community_creator(data.id, &local_user_view, &context).await?;
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  MultiCommunity::delete_entry(&mut context.pool(), data.id, &community).await?;

  if !community.local {
    let used_in_multiple =
      MultiCommunity::community_used_in_multiple(&mut context.pool(), multi.id, community.id)
        .await?;
    // unfollow the community only if its not used in another multi-community
    if !used_in_multiple {
      let multicomm_follower = SiteView::read_multicomm_follower(&mut context.pool()).await?;
      CommunityActions::unfollow(&mut context.pool(), multicomm_follower.id, community.id).await?;
      ActivityChannel::submit_activity(
        SendActivityData::FollowCommunity(community, local_user_view.person.clone(), false),
        &context,
      )?;
    }
  }

  send_federation_update(multi, local_user_view, &context).await?;

  Ok(Json(SuccessResponse::default()))
}
