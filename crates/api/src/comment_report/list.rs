use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  comment::{ListCommentReports, ListCommentReportsResponse},
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_views::comment_report_view::CommentReportQuery;
use lemmy_utils::error::LemmyError;

/// Lists comment reports for a community if an id is supplied
/// or returns all comment reports for communities a user moderates
#[tracing::instrument(skip(context))]
pub async fn list_comment_reports(
  data: Query<ListCommentReports>,
  context: Data<LemmyContext>,
) -> Result<Json<ListCommentReportsResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let community_id = data.community_id;
  let unresolved_only = data.unresolved_only;

  let page = data.page;
  let limit = data.limit;
  let comment_reports = CommentReportQuery {
    community_id,
    unresolved_only,
    page,
    limit,
  }
  .list(&mut context.pool(), &local_user_view.person)
  .await?;

  Ok(Json(ListCommentReportsResponse { comment_reports }))
}
