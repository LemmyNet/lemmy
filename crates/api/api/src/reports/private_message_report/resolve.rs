use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{source::private_message_report::PrivateMessageReport, traits::Reportable};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_reports::{
  api::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
  PrivateMessageReportView,
};
use lemmy_utils::error::LemmyResult;

pub async fn resolve_pm_report(
  data: Json<ResolvePrivateMessageReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  if data.resolved {
    PrivateMessageReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    PrivateMessageReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let private_message_report_view =
    PrivateMessageReportView::read(&mut context.pool(), report_id).await?;

  Ok(Json(PrivateMessageReportResponse {
    private_message_report_view,
  }))
}
