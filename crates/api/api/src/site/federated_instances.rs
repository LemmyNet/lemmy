use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::build_federated_instances};
use lemmy_db_views_site::{api::GetFederatedInstancesResponse, SiteView};
use lemmy_utils::error::LemmyResult;

pub async fn get_federated_instances(
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetFederatedInstancesResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let federated_instances =
    build_federated_instances(&site_view.local_site, &mut context.pool()).await?;

  Ok(Json(GetFederatedInstancesResponse {
    federated_instances,
  }))
}
