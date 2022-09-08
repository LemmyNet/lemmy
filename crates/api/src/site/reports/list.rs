use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{ListReports, ListReportsResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_views::{
  comment_report_view::CommentReportQuery,
  post_report_view::PostReportQuery,
  private_message_report_view::PrivateMessageReportQuery,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

/// Lists post reports for a community if an id is supplied
/// or returns all post reports for communities a user moderates
///
/// If called by an admin, private message reports are also included.
#[async_trait::async_trait(?Send)]
impl Perform for ListReports {
  type Response = ListReportsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&self.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let admin = local_user_view.person.admin;
    let community_id = self.community_id;
    let unresolved_only = self.unresolved_only;
    let page = self.page;
    let limit = self.limit;

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

    let post_reports = blocking(context.pool(), move |conn| {
      PostReportQuery::builder()
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

    let private_message_reports = if admin {
      blocking(context.pool(), move |conn| {
        PrivateMessageReportQuery::builder()
          .conn(conn)
          .unresolved_only(unresolved_only)
          .page(page)
          .limit(limit)
          .build()
          .list()
      })
      .await??
    } else {
      vec![]
    };

    Ok(ListReportsResponse {
      comment_reports,
      post_reports,
      private_message_reports,
    })
  }
}
