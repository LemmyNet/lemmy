// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{GetSiteMetadata, GetSiteMetadataResponse},
  request::fetch_site_metadata,
};
use lemmy_utils::{ConnectionId, LemmyError};
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
