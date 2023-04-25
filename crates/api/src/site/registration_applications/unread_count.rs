use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  sensitive::Sensitive,
  site::{GetUnreadRegistrationApplicationCount, GetUnreadRegistrationApplicationCountResponse},
  utils::{is_admin, local_user_view_from_jwt_new},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views::structs::RegistrationApplicationView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for GetUnreadRegistrationApplicationCount {
  type Response = GetUnreadRegistrationApplicationCountResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let _data = self;
    let local_user_view = local_user_view_from_jwt_new(auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    // Only let admins do this
    is_admin(&local_user_view)?;

    let verified_email_only = local_site.require_email_verification;

    let registration_applications =
      RegistrationApplicationView::get_unread_count(context.pool(), verified_email_only).await?;

    Ok(Self::Response {
      registration_applications,
    })
  }
}
