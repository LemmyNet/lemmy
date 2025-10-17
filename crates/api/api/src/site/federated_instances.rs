use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_site::{
  api::{GetFederatedInstances, GetFederatedInstancesResponse},
  FederatedInstancesView,
};
use lemmy_utils::error::LemmyResult;

pub async fn get_federated_instances(
  data: Query<GetFederatedInstances>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetFederatedInstancesResponse>> {
  let federated_instances =
    FederatedInstancesView::list(&mut context.pool(), data.into_inner()).await?;

  Ok(Json(GetFederatedInstancesResponse {
    federated_instances,
  }))
}
