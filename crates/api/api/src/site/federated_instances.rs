use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_site::{api::GetFederatedInstancesResponse, FederatedInstancesView};
use lemmy_utils::error::LemmyResult;

pub async fn get_federated_instances(
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetFederatedInstancesResponse>> {
  let federated_instances = FederatedInstancesView::list(&mut context.pool()).await?;

  Ok(Json(GetFederatedInstancesResponse {
    federated_instances,
  }))
}
