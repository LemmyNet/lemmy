use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BannedPersonsResponse, GetBannedPersons},
  sensitive::Sensitive,
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for GetBannedPersons {
  type Response = BannedPersonsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let _data: &GetBannedPersons = self;
    let local_user_view = local_user_view_from_jwt(auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let banned = PersonView::banned(context.pool()).await?;

    let res = Self::Response { banned };

    Ok(res)
  }
}
