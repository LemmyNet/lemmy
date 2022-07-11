// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{GetSiteConfigResponse, SaveSiteConfig},
  utils::{get_local_user_view_from_jwt, is_admin},
};
use lemmy_utils::{settings::structs::Settings, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for SaveSiteConfig {
  type Response = GetSiteConfigResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteConfigResponse, LemmyError> {
    let data: &SaveSiteConfig = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins read this
    is_admin(&local_user_view)?;

    // Make sure docker doesn't have :ro at the end of the volume, so its not a read-only filesystem
    let config_hjson = Settings::save_config_file(&data.config_hjson)
      .map_err(|e| e.with_message("couldnt_update_site"))?;

    Ok(GetSiteConfigResponse { config_hjson })
  }
}
