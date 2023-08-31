use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{ListPrivateMessageReports, ListPrivateMessageReportsResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_views::private_message_report_view::PrivateMessageReportQuery;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_pm_reports(
  data: Json<ListPrivateMessageReports>,
  context: Data<LemmyContext>,
) -> Result<Json<ListPrivateMessageReportsResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

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
