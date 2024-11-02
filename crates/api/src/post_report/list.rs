use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{ListPostReports, ListPostReportsResponse},
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_views::{post_report_view::PostReportQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

/// Lists post reports for a community if an id is supplied
/// or returns all post reports for communities a user moderates
#[tracing::instrument(skip(context))]
pub async fn list_post_reports(
  data: Query<ListPostReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPostReportsResponse>> {
  let community_id = data.community_id;
  let post_id = data.post_id;
  let unresolved_only = data.unresolved_only.unwrap_or_default();

  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;

  let page = data.page;
  let limit = data.limit;
  let post_reports = PostReportQuery {
    community_id,
    post_id,
    unresolved_only,
    page,
    limit,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(ListPostReportsResponse { post_reports }))
}
