use crate::multi_community::get_multi;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::newtypes::MultiCommunityId;
use lemmy_db_views_community::api::GetMultiCommunityResponse;
use lemmy_utils::error::LemmyResult;

pub async fn get_multi_community(
  id: Path<MultiCommunityId>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  get_multi(id.into_inner(), context).await
}
