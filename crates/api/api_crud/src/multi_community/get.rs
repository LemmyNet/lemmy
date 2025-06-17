use crate::multi_community::get_multi;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_community::api::{GetMultiCommunity, GetMultiCommunityResponse};
use lemmy_utils::error::LemmyResult;

pub async fn get_multi_community(
  data: Query<GetMultiCommunity>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  get_multi(data.id, context).await
}
