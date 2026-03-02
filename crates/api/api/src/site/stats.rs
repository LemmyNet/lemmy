use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_site::{
  FederatedInstanceView,
  api::{GetFederatedInstances, GetFederatedInstancesKind, GetSiteStatsResponse},
};
use lemmy_utils::{CacheLock, build_cache, error::LemmyResult};
// use lemmy_db_views_site::SiteView;
use std::sync::LazyLock;

pub async fn get_stats(context: Data<LemmyContext>) -> LemmyResult<Json<GetSiteStatsResponse>> {
  static CACHE: CacheLock<GetSiteStatsResponse> = LazyLock::new(build_cache);

  let stats_response = Box::pin(CACHE.try_get_with((), build_stats(&context)))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to construct site stats: {e}"))?;

  Ok(Json(stats_response))
}

async fn build_stats(context: &LemmyContext) -> LemmyResult<GetSiteStatsResponse> {
  let data = GetFederatedInstances {
    domain_filter: None,
    kind: GetFederatedInstancesKind::Linked,
    page_cursor: None,
    limit: None,
  };

  let linked_instances = FederatedInstanceView::count(&mut context.pool(), data).await?;
  Ok(GetSiteStatsResponse { linked_instances })
}
