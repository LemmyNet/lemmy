use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetFederatedInstances, GetFederatedInstancesResponse},
  utils::build_federated_instances,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for GetFederatedInstances {
  type Response = GetFederatedInstancesResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let site_view = SiteView::read_local(context.pool()).await?;
    let federated_instances =
      build_federated_instances(&site_view.local_site, context.pool()).await?;

    Ok(Self::Response {
      federated_instances,
    })
  }
}
