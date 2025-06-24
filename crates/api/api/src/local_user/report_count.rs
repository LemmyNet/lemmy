use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_community_mod_of_any_or_admin_action};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::ReportCombinedViewInternal;
use lemmy_db_views_reports::api::{GetReportCount, GetReportCountResponse};
use lemmy_utils::error::LemmyResult;

pub async fn report_count(
  data: Query<GetReportCount>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetReportCountResponse>> {
  check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;

  let count = ReportCombinedViewInternal::get_report_count(
    &mut context.pool(),
    &local_user_view,
    data.community_id,
  )
  .await?;

  Ok(Json(GetReportCountResponse { count }))
}
