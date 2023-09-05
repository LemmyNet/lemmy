use actix_web::web::{Data, Json};
use lemmy_api_common::{
    context::LemmyContext, site::GetFederatedInstancesResponse, utils::build_federated_instances,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn get_federated_instances(
    context: Data<LemmyContext>,
) -> Result<Json<GetFederatedInstancesResponse>, LemmyError> {
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let federated_instances =
        build_federated_instances(&site_view.local_site, &mut context.pool()).await?;

    Ok(Json(GetFederatedInstancesResponse {
        federated_instances,
    }))
}
