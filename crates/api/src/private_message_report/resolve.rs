use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
  utils::is_admin,
};
use lemmy_db_schema::{source::private_message_report::PrivateMessageReport, traits::Reportable};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageReportView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn resolve_pm_report(
  data: Json<ResolvePrivateMessageReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  if data.resolved {
    PrivateMessageReport::resolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  } else {
    PrivateMessageReport::unresolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  }

  let private_message_report_view = PrivateMessageReportView::read(&mut context.pool(), report_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPrivateMessageReport)?;

  Ok(Json(PrivateMessageReportResponse {
    private_message_report_view,
  }))
}
