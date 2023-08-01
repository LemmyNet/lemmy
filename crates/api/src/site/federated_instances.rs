use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetFederatedInstances, GetFederatedInstancesResponse},
  utils::build_federated_instances,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for GetFederatedInstances {
  type Response = GetFederatedInstancesResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let federated_instances =
      build_federated_instances(&site_view.local_site, &mut context.pool()).await?;

    Ok(Self::Response {
      federated_instances,
    })
  }
}
