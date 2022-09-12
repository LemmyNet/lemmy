use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{ListCommentReports, ListCommentReportsResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_views::comment_report_view::CommentReportQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

/// Lists comment reports for a community if an id is supplied
/// or returns all comment reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListCommentReports {
  type Response = ListCommentReportsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommentReportsResponse, LemmyError> {
    let data: &ListCommentReports = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let admin = local_user_view.person.admin;
    let community_id = data.community_id;
    let unresolved_only = data.unresolved_only;

    let page = data.page;
    let limit = data.limit;
    let comment_reports = blocking(context.pool(), move |conn| {
      CommentReportQuery::builder()
        .conn(conn)
        .my_person_id(person_id)
        .admin(admin)
        .community_id(community_id)
        .unresolved_only(unresolved_only)
        .page(page)
        .limit(limit)
        .build()
        .list()
    })
    .await??;

    let res = ListCommentReportsResponse { comment_reports };

    Ok(res)
  }
}
