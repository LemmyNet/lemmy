use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetSiteMetadata, GetSiteMetadataResponse},
  request::fetch_site_metadata,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for GetSiteMetadata {
  type Response = GetSiteMetadataResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<GetSiteMetadataResponse, LemmyError> {
    let data: &Self = self;

    let metadata = fetch_site_metadata(context.client(), &data.url).await?;

    Ok(GetSiteMetadataResponse { metadata })
  }
}
