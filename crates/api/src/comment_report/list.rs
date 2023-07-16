use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{ListCommentReports, ListCommentReportsResponse},
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_views::comment_report_view::CommentReportQuery;
use lemmy_utils::error::LemmyError;

/// Lists comment reports for a community if an id is supplied
/// or returns all comment reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListCommentReports {
  type Response = ListCommentReportsResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<ListCommentReportsResponse, LemmyError> {
    let data: &ListCommentReports = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

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

    Ok(ListCommentReportsResponse { comment_reports })
  }
}
