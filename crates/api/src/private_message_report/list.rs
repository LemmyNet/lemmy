use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  private_message::{ListPrivateMessageReports, ListPrivateMessageReportsResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_views::private_message_report_view::PrivateMessageReportQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for ListPrivateMessageReports {
  type Response = ListPrivateMessageReportsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&self.auth, context.pool(), context.secret()).await?;

    is_admin(&local_user_view)?;

    let unresolved_only = self.unresolved_only;
    let page = self.page;
    let limit = self.limit;
    let private_message_reports = blocking(context.pool(), move |conn| {
      PrivateMessageReportQuery::builder()
        .conn(conn)
        .unresolved_only(unresolved_only)
        .page(page)
        .limit(limit)
        .build()
        .list()
    })
    .await??;

    let res = ListPrivateMessageReportsResponse {
      private_message_reports,
    };

    Ok(res)
  }
}
