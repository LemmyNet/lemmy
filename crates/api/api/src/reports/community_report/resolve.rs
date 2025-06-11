use activitypub_federation::config::Data;
use actix_web::web::Json;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{community_report::CommunityReport, site::Site},
  traits::Reportable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_reports::{
  api::{CommunityReportResponse, ResolveCommunityReport},
  CommunityReportView,
};
use lemmy_utils::error::LemmyResult;

pub async fn resolve_community_report(
  data: Json<ResolveCommunityReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  if data.resolved {
    CommunityReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    CommunityReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let community_report_view =
    CommunityReportView::read(&mut context.pool(), report_id, person_id).await?;
  let site = Site::read_from_instance_id(
    &mut context.pool(),
    community_report_view.community.instance_id,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: community_report_view.community.ap_id.inner().clone(),
      actor: local_user_view.person,
      report_creator: community_report_view.creator.clone(),
      receiver: Either::Left(site),
    },
    &context,
  )?;

  Ok(Json(CommunityReportResponse {
    community_report_view,
  }))
}
