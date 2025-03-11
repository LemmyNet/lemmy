use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  reports::community::{CommunityReportResponse, ResolveCommunityReport},
  utils::is_admin,
};
use lemmy_db_schema::{source::community_report::CommunityReport, traits::Reportable};
use lemmy_db_views::structs::{CommunityReportView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

pub async fn resolve_community_report(
  data: Json<ResolveCommunityReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  if data.resolved {
    CommunityReport::resolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  } else {
    CommunityReport::unresolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  }

  let community_report_view =
    CommunityReportView::read(&mut context.pool(), report_id, person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: community_report_view.community.ap_id.inner().clone(),
      actor: local_user_view.person,
      report_creator: report.creator,
      community: SiteOrCommunity::Site(todo!()),
    },
    &context,
  )?;

  Ok(Json(CommunityReportResponse {
    community_report_view,
  }))
}
