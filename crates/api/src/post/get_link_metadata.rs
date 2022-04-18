use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::post::{GetSiteMetadata, GetSiteMetadataResponse};
use lemmy_utils::{request::fetch_site_metadata, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetSiteMetadata {
  type Response = GetSiteMetadataResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteMetadataResponse, LemmyError> {
    let data: &Self = self;

    let metadata = fetch_site_metadata(context.client(), &data.url).await?;

    Ok(GetSiteMetadataResponse { metadata })
  }
}
