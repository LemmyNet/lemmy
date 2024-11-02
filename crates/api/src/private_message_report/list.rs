use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{ListPrivateMessageReports, ListPrivateMessageReportsResponse},
  utils::is_admin,
};
use lemmy_db_views::{
  private_message_report_view::PrivateMessageReportQuery,
  structs::LocalUserView,
};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn list_pm_reports(
  data: Query<ListPrivateMessageReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPrivateMessageReportsResponse>> {
  is_admin(&local_user_view)?;

  let unresolved_only = data.unresolved_only.unwrap_or_default();
  let page = data.page;
  let limit = data.limit;
  let private_message_reports = PrivateMessageReportQuery {
    unresolved_only,
    page,
    limit,
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(ListPrivateMessageReportsResponse {
    private_message_reports,
  }))
}
