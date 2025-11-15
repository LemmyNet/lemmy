use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_site::{
  FederatedInstanceView,
  api::{GetFederatedInstances, GetFederatedInstancesResponse},
};
use lemmy_utils::error::LemmyResult;

pub async fn get_federated_instances(
  data: Query<GetFederatedInstances>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetFederatedInstancesResponse>> {
  let federated_instances =
    FederatedInstanceView::list(&mut context.pool(), data.into_inner()).await?;

  let next_page = federated_instances
    .last()
    .map(PaginationCursorBuilder::to_cursor);
  let prev_page = federated_instances
    .first()
    .map(PaginationCursorBuilder::to_cursor);

  // Return the jwt
  Ok(Json(GetFederatedInstancesResponse {
    federated_instances,
    next_page,
    prev_page,
  }))
}
