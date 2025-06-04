use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  community::{GetMultiCommunity, GetMultiCommunityResponse},
  context::LemmyContext,
};
use lemmy_db_schema::{source::multi_community::MultiCommunity, traits::Crud};
use lemmy_db_views_community::impls::CommunityQuery;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_multi_community(
  data: Query<GetMultiCommunity>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;
  let multi = MultiCommunity::read(&mut context.pool(), data.id).await?;
  let communities = CommunityQuery {
    multi_community_id: Some(multi.id),
    ..Default::default()
  }
  .list(&local_site.site, &mut context.pool())
  .await?;
  Ok(Json(GetMultiCommunityResponse { multi, communities }))
}
