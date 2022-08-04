use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{ListRegistrationApplications, ListRegistrationApplicationsResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::source::site::Site;
use lemmy_db_views::registration_application_view::RegistrationApplicationQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

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

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let unread_only = data.unread_only;
    let verified_email_only = blocking(context.pool(), Site::read_local_site)
      .await??
      .require_email_verification;

    let page = data.page;
    let limit = data.limit;
    let registration_applications = blocking(context.pool(), move |conn| {
      RegistrationApplicationQuery::builder()
        .conn(conn)
        .unread_only(unread_only)
        .verified_email_only(Some(verified_email_only))
        .page(page)
        .limit(limit)
        .build()
        .list()
    })
    .await??;

    let res = Self::Response {
      registration_applications,
    };

    Ok(res)
  }
}
