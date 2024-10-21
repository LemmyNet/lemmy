// SPDX-FileCopyrightText: 2019 Lemmy Developers
//
// SPDX-License-Identifier: AGPL-3.0-only

use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetSiteMetadata, GetSiteMetadataResponse},
  request::fetch_link_metadata,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyResult},
  LemmyErrorType,
};
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn get_link_metadata(
  data: Query<GetSiteMetadata>,
  context: Data<LemmyContext>,
  // Require an account for this API
  _local_user_view: LocalUserView,
) -> LemmyResult<Json<GetSiteMetadataResponse>> {
  let url = Url::parse(&data.url).with_lemmy_type(LemmyErrorType::InvalidUrl)?;
  let metadata = fetch_link_metadata(&url, &context).await?;

  Ok(Json(GetSiteMetadataResponse { metadata }))
}
