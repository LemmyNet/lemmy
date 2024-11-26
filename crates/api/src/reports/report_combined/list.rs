use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  reports::combined::{ListReports, ListReportsResponse},
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_views::{report_combined_view::ReportCombinedQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

/// Lists reports for a community if an id is supplied
/// or returns all reports for communities a user moderates
#[tracing::instrument(skip(context))]
pub async fn list_reports(
  data: Query<ListReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListReportsResponse>> {
  let community_id = data.community_id;
  let unresolved_only = data.unresolved_only.unwrap_or_default();

  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;

  let page = data.page;
  let limit = data.limit;
  let reports = ReportCombinedQuery {
    community_id,
    unresolved_only,
    page,
    limit,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(ListReportsResponse { reports }))
}
