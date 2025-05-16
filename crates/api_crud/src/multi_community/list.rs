use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  community::{ListMultiCommunities, ListMultiCommunitiesResponse},
  context::LemmyContext,
};
use lemmy_db_schema::source::multi_community::MultiCommunity;
use lemmy_utils::error::LemmyResult;

pub async fn list_multi_communities(
  data: Query<ListMultiCommunities>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<ListMultiCommunitiesResponse>> {
  let res = MultiCommunity::list(&mut context.pool(), data.owner_id).await?;
  Ok(Json(ListMultiCommunitiesResponse {
    multi_communities: res,
  }))
}
