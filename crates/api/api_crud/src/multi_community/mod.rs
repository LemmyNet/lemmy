use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  newtypes::MultiCommunityId,
  source::multi_community::MultiCommunity,
  traits::Crud,
};
use lemmy_db_views_community::{
  api::GetMultiCommunityResponse,
  impls::CommunityQuery,
  MultiCommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub mod create;
pub mod create_entry;
pub mod delete_entry;
pub mod get;
pub mod list;
pub mod update;

/// Check that current user is creator of multi-comm and can modify it.
async fn check_multi_community_creator(
  id: MultiCommunityId,
  local_user_view: &LocalUserView,
  context: &LemmyContext,
) -> LemmyResult<MultiCommunity> {
  let multi = MultiCommunity::read(&mut context.pool(), id).await?;
  if multi.local && local_user_view.local_user.admin {
    return Ok(multi);
  }
  if multi.creator_id != local_user_view.person.id {
    return Err(LemmyErrorType::MultiCommunityUpdateWrongUser.into());
  }
  Ok(multi)
}

async fn send_federation_update(
  multi: MultiCommunity,
  local_user_view: LocalUserView,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  ActivityChannel::submit_activity(
    SendActivityData::UpdateMultiCommunity(multi, local_user_view.person),
    context,
  )?;
  Ok(())
}

async fn get_multi(
  id: MultiCommunityId,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;
  let multi_community_view = MultiCommunityView::read(&mut context.pool(), id).await?;
  let communities = CommunityQuery {
    multi_community_id: Some(multi_community_view.multi.id),
    ..Default::default()
  }
  .list(&local_site.site, &mut context.pool())
  .await?;
  Ok(Json(GetMultiCommunityResponse {
    multi_community_view,
    communities,
  }))
}
