use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ListRegistrationApplications, ListRegistrationApplicationsResponse},
  utils::{get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views::registration_application_view::RegistrationApplicationQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};

/// Lists registration applications, filterable by undenied only.
#[async_trait::async_trait(?Send)]
impl Perform for ListRegistrationApplications {
  type Response = ListRegistrationApplicationsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let unread_only = data.unread_only;
    let verified_email_only = local_site.require_email_verification;

    let page = data.page;
    let limit = data.limit;
    let registration_applications = RegistrationApplicationQuery::builder()
      .pool(context.pool())
      .unread_only(unread_only)
      .verified_email_only(Some(verified_email_only))
      .page(page)
      .limit(limit)
      .build()
      .list()
      .await?;

    let res = Self::Response {
      registration_applications,
    };

    Ok(res)
  }
}
