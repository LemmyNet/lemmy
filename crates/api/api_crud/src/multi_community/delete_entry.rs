use super::{check_multi_community_creator, send_federation_update};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_local_user_valid,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions},
    multi_community::{MultiCommunity, MultiCommunityEntry, MultiCommunityEntryForm},
  },
  traits::Followable,
};
use lemmy_db_views_community::api::CreateOrDeleteMultiCommunityEntry;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{SiteView, api::SuccessResponse};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn delete_multi_community_entry(
  data: Json<CreateOrDeleteMultiCommunityEntry>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  check_local_user_valid(&local_user_view)?;

  let multi = MultiCommunity::read(&mut context.pool(), data.id).await?;
  check_multi_community_creator(&multi, &local_user_view)?;

  let community = Community::read(&mut context.pool(), data.community_id).await?;

  let form = MultiCommunityEntryForm {
    multi_community_id: data.id,
    community_id: data.community_id,
  };
  MultiCommunityEntry::delete(&mut context.pool(), &form).await?;

  if !community.local {
    let used_in_multiple =
      MultiCommunityEntry::community_used_in_multiple(&mut context.pool(), &form).await?;
    // unfollow the community only if its not used in another multi-community
    if !used_in_multiple {
      let multicomm_follower = SiteView::read_system_account(&mut context.pool()).await?;
      CommunityActions::unfollow(&mut context.pool(), multicomm_follower.id, community.id).await?;
      ActivityChannel::submit_activity(
        SendActivityData::FollowCommunity(community, local_user_view.person.clone(), false),
        &context,
      )?;
    }
  }

  send_federation_update(multi, local_user_view.person, &context)?;

  Ok(Json(SuccessResponse::default()))
}
