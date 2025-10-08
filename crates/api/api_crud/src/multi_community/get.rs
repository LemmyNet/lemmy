use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_community::{
  api::{GetMultiCommunity, GetMultiCommunityResponse},
  impls::CommunityQuery,
  MultiCommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_multi_community(
  data: Query<GetMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;
  let my_person_id = local_user_view.map(|l| l.person.id);

  let multi_community_view =
    MultiCommunityView::read(&mut context.pool(), data.id, my_person_id).await?;

  let communities = CommunityQuery {
    multi_community_id: Some(data.id),
    ..Default::default()
  }
  .list(&local_site.site, &mut context.pool())
  .await?;

  Ok(Json(GetMultiCommunityResponse {
    multi_community_view,
    communities,
  }))
}
