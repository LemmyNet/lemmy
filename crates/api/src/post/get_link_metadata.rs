use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetSiteMetadata, GetSiteMetadataResponse},
  request::fetch_link_metadata,
};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn get_link_metadata(
  data: Query<GetSiteMetadata>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteMetadataResponse>> {
  let metadata = fetch_link_metadata(&data.url, false, &context).await?;

  Ok(Json(GetSiteMetadataResponse { metadata }))
}
