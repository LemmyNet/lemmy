use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_community_mod_of_any_or_admin_action};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  ReportCombinedView,
  api::ListReports,
  impls::ReportCombinedQuery,
};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

/// Lists reports for a community if an id is supplied
/// or returns all reports for communities a user moderates
pub async fn list_reports(
  Query(data): Query<ListReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<ReportCombinedView>>> {
  let my_reports_only = data.my_reports_only;

  // Only check mod or admin status when not viewing my reports
  if !my_reports_only.unwrap_or_default() {
    check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;
  }

  let reports = ReportCombinedQuery {
    community_id: data.community_id,
    post_id: data.post_id,
    type_: data.type_,
    unresolved_only: data.unresolved_only,
    show_community_rule_violations: data.show_community_rule_violations,
    my_reports_only,
    page_cursor: data.page_cursor,
    limit: data.limit,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(reports))
}
