use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  comment::{ListCommentReports, ListCommentReportsResponse},
  context::LemmyContext,
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_views::{comment_report_view::CommentReportQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

/// Lists comment reports for a community if an id is supplied
/// or returns all comment reports for communities a user moderates
#[tracing::instrument(skip(context))]
pub async fn list_comment_reports(
  data: Query<ListCommentReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListCommentReportsResponse>> {
  let community_id = data.community_id;
  let comment_id = data.comment_id;
  let unresolved_only = data.unresolved_only.unwrap_or_default();

  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;

  let page = data.page;
  let limit = data.limit;
  let comment_reports = CommentReportQuery {
    community_id,
    comment_id,
    unresolved_only,
    page,
    limit,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(ListCommentReportsResponse { comment_reports }))
}
