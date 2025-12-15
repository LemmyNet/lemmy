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
use lemmy_db_views_site::{SiteView, api::UnreadCountsResponse};
use lemmy_utils::error::LemmyResult;

pub async fn get_unread_counts(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<UnreadCountsResponse>> {
  let person = &local_user_view.person;
  let mut res = UnreadCountsResponse::default();

  let show_bot_accounts = local_user_view.local_user.show_bot_accounts;
  res.notification_count =
    NotificationView::get_unread_count(&mut context.pool(), person, show_bot_accounts).await?;

  // Community mods get additional counts for reports and pending follows for private communities.
  if check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool())
    .await
    .is_ok()
  {
    res.report_count = Some(
      ReportCombinedViewInternal::get_report_count(&mut context.pool(), &local_user_view).await?,
    );
    res.pending_follow_count =
      Some(PendingFollowerView::count_approval_required(&mut context.pool(), person.id).await?);
  }

  // Admins also get the number of unread registration applications.
  if is_admin(&local_user_view).is_ok() {
    let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
    let verified_email_only = local_site.require_email_verification;
    res.registration_application_count = Some(
      RegistrationApplicationView::get_unread_count(&mut context.pool(), verified_email_only)
        .await?,
    )
  }

  Ok(Json(res))
}
