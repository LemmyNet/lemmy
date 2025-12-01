use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_site::{FederatedInstanceView, api::GetFederatedInstances};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn get_federated_instances(
  Query(data): Query<GetFederatedInstances>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<PagedResponse<FederatedInstanceView>>> {
  let federated_instances = FederatedInstanceView::list(&mut context.pool(), data).await?;

  // Return the jwt
  Ok(Json(federated_instances))
}
