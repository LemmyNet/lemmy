use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_community_mod_of_any_or_admin_action, is_admin},
};
use lemmy_db_views_community_follower_approval::PendingFollowerView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::NotificationView;
use lemmy_db_views_registration_applications::RegistrationApplicationView;
use lemmy_db_views_report_combined::ReportCombinedViewInternal;
use lemmy_db_views_site::{SiteView, api::CountsResponse};
use lemmy_utils::error::LemmyResult;

pub async fn get_counts(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CountsResponse>> {
  let person = &local_user_view.person;
  let mut res = CountsResponse::default();

  let show_bot_accounts = local_user_view.local_user.show_bot_accounts;
  res.notification_count =
    NotificationView::get_unread_count(&mut context.pool(), person, show_bot_accounts).await?;

  if check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool())
    .await
    .is_ok()
  {
    res.pending_follow_count =
      PendingFollowerView::count_approval_required(&mut context.pool(), person.id).await?;
    res.report_count =
      ReportCombinedViewInternal::get_report_count(&mut context.pool(), &local_user_view).await?;
  }

  if is_admin(&local_user_view).is_ok() {
    let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
    let verified_email_only = local_site.require_email_verification;
    res.registration_application_count =
      RegistrationApplicationView::get_unread_count(&mut context.pool(), verified_email_only)
        .await?
  }

  Ok(Json(res))
}
