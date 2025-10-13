use super::{check_multi_community_creator, send_federation_update};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_community_response,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_deleted_removed, check_local_user_valid},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions, CommunityFollowerForm},
    multi_community::{MultiCommunity, MultiCommunityEntry, MultiCommunityEntryForm},
  },
  traits::{Crud, Followable},
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_community::api::{CommunityResponse, CreateOrDeleteMultiCommunityEntry};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn create_multi_community_entry(
  data: Json<CreateOrDeleteMultiCommunityEntry>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  let community_id = data.community_id;

  check_local_user_valid(&local_user_view)?;

  let multi = MultiCommunity::read(&mut context.pool(), data.id).await?;
  check_multi_community_creator(&multi, &local_user_view)?;

  let community = Community::read(&mut context.pool(), community_id).await?;
  check_community_deleted_removed(&community)?;

  MultiCommunityEntry::check_entry_limit(&mut context.pool(), data.id).await?;

  let form = MultiCommunityEntryForm {
    multi_community_id: data.id,
    community_id,
  };
  let inserted_entry = MultiCommunityEntry::create(&mut context.pool(), &form).await?;

  if !community.local {
    let multicomm_follower = SiteView::read_multicomm_follower(&mut context.pool()).await?;
    let actions = CommunityActions::read(&mut context.pool(), community.id, multicomm_follower.id)
      .await
      .unwrap_or_default();

    // follow the community if not already followed
    if actions.followed_at.is_none() {
      let form = CommunityFollowerForm::new(
        community.id,
        multicomm_follower.id,
        CommunityFollowerState::Pending,
      );
      CommunityActions::follow(&mut context.pool(), &form).await?;
      ActivityChannel::submit_activity(
        SendActivityData::FollowCommunity(community, local_user_view.person.clone(), true),
        &context,
      )?;
    }
  }

  send_federation_update(multi, local_user_view.person.clone(), &context)?;

  build_community_response(&context, local_user_view, inserted_entry.community_id).await
}
