use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{ListPostReports, ListPostReportsResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_views::post_report_view::PostReportQuery;
use lemmy_utils::error::LemmyError;

/// Lists post reports for a community if an id is supplied
/// or returns all post reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListPostReports {
  type Response = ListPostReportsResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<ListPostReportsResponse, LemmyError> {
    let data: &ListPostReports = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let community_id = data.community_id;
    let unresolved_only = data.unresolved_only.unwrap_or_default();

    let page = data.page;
    let limit = data.limit;
    let post_reports = PostReportQuery {
      community_id,
      unresolved_only,
      page,
      limit,
    }
    .list(&mut context.pool(), &local_user_view)
    .await?;

    Ok(ListPostReportsResponse { post_reports })
  }
}
