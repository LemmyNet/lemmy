use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{ListRegistrationApplications, ListRegistrationApplicationsResponse},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{local_site::LocalSite, registration_application::RegistrationApplication},
  traits::PageCursorBuilder,
};
use lemmy_db_views::{
  registration_applications::registration_application_view::RegistrationApplicationQuery,
  structs::LocalUserView,
};
use lemmy_utils::error::LemmyResult;

/// Lists registration applications, filterable by undenied only.
pub async fn list_registration_applications(
  data: Query<ListRegistrationApplications>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListRegistrationApplicationsResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(RegistrationApplication::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let registration_applications = RegistrationApplicationQuery {
    unread_only: data.unread_only,
    verified_email_only: Some(local_site.require_email_verification),
    cursor_data,
    page_back: data.page_back,
  }
  .list(&mut context.pool())
  .await?;

  let next_page = registration_applications
    .last()
    .map(PageCursorBuilder::cursor);

  Ok(Json(ListRegistrationApplicationsResponse {
    registration_applications,
    next_page,
  }))
}
