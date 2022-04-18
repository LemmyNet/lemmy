use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  post::{ListPostReports, ListPostReportsResponse},
};
use lemmy_db_views::post_report_view::PostReportQueryBuilder;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

/// Lists post reports for a community if an id is supplied
/// or returns all post reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListPostReports {
  type Response = ListPostReportsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListPostReportsResponse, LemmyError> {
    let data: &ListPostReports = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let admin = local_user_view.person.admin;
    let community_id = data.community_id;
    let unresolved_only = data.unresolved_only;

    let page = data.page;
    let limit = data.limit;
    let post_reports = blocking(context.pool(), move |conn| {
      PostReportQueryBuilder::create(conn, person_id, admin)
        .community_id(community_id)
        .unresolved_only(unresolved_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    let res = ListPostReportsResponse { post_reports };

    Ok(res)
  }
}
