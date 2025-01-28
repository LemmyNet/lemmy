use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  reports::combined::{ListReports, ListReportsResponse},
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_views::{combined::report_combined_view::ReportCombinedQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

/// Lists reports for a community if an id is supplied
/// or returns all reports for communities a user moderates
#[tracing::instrument(skip(context))]
pub async fn list_reports(
  data: Query<ListReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListReportsResponse>> {
  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let reports = ReportCombinedQuery {
    community_id: data.community_id,
    post_id: data.post_id,
    type_: data.type_,
    unresolved_only: data.unresolved_only,
    page_after,
    page_back,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(ListReportsResponse { reports }))
}
