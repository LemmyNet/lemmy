use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_views_inbox_combined::api::GetUnreadRegistrationApplicationCountResponse;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_registration_applications::RegistrationApplicationView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_unread_registration_application_count(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetUnreadRegistrationApplicationCountResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  // Only let admins do this
  is_admin(&local_user_view)?;

  let verified_email_only = local_site.require_email_verification;

  let registration_applications =
    RegistrationApplicationView::get_unread_count(&mut context.pool(), verified_email_only).await?;

  Ok(Json(GetUnreadRegistrationApplicationCountResponse {
    registration_applications,
  }))
}
