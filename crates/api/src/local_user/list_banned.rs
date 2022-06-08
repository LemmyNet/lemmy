use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{BannedPersonsResponse, GetBannedPersons},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_views_actor::structs::PersonViewSafe;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetBannedPersons {
  type Response = BannedPersonsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &GetBannedPersons = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let banned = blocking(context.pool(), PersonViewSafe::banned).await??;

    let res = Self::Response { banned };

    Ok(res)
  }
}
