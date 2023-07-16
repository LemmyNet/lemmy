use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{ListPrivateMessageReports, ListPrivateMessageReportsResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_views::private_message_report_view::PrivateMessageReportQuery;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for ListPrivateMessageReports {
  type Response = ListPrivateMessageReportsResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&self.auth, context).await?;

    is_admin(&local_user_view)?;

    let unresolved_only = self.unresolved_only;
    let page = self.page;
    let limit = self.limit;
    let private_message_reports = PrivateMessageReportQuery {
      unresolved_only,
      page,
      limit,
    }
    .list(&mut context.pool())
    .await?;

    Ok(ListPrivateMessageReportsResponse {
      private_message_reports,
    })
  }
}
