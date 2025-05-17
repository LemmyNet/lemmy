use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetReportCount, GetReportCountResponse},
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_views::structs::{LocalUserView, ReportCombinedViewInternal};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
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
