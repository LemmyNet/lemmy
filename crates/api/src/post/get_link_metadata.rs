use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, request::fetch_link_metadata};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{GetSiteMetadata, GetSiteMetadataResponse};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use url::Url;

pub async fn get_link_metadata(
  data: Query<GetSiteMetadata>,
  context: Data<LemmyContext>,
  // Require an account for this API
  _local_user_view: LocalUserView,
) -> LemmyResult<Json<GetSiteMetadataResponse>> {
  let url = Url::parse(&data.url).with_lemmy_type(LemmyErrorType::InvalidUrl)?;
  let metadata = fetch_link_metadata(&url, &context, false).await?;

  Ok(Json(GetSiteMetadataResponse { metadata }))
}
