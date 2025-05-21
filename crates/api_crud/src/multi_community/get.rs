use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{community::GetMultiCommunity, context::LemmyContext};
use lemmy_db_views_community::{multi_community::ReadParams, MultiCommunityView};
use lemmy_utils::error::LemmyResult;

pub async fn get_multi_community(
  data: Query<GetMultiCommunity>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<MultiCommunityView>> {
  let multi = MultiCommunityView::read(&mut context.pool(), ReadParams::Id(data.id)).await?;

  Ok(Json(multi))
}
