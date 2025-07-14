use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{source::private_message_report::PrivateMessageReport, traits::Reportable};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  api::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
  ReportCombinedViewInternal,
};
use lemmy_utils::error::LemmyResult;

pub async fn resolve_pm_report(
  data: Json<ResolvePrivateMessageReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person = &local_user_view.person;
  PrivateMessageReport::update_resolved(&mut context.pool(), report_id, person.id, data.resolved)
    .await?;

  let private_message_report_view =
    ReportCombinedViewInternal::read_private_message_report(&mut context.pool(), report_id, person)
      .await?;

  Ok(Json(PrivateMessageReportResponse {
    private_message_report_view,
  }))
}
