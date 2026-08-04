use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{
    community_report::{CommunityReport, UpdateCommunityReportForm},
    site::Site,
  },
  traits::Reportable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  ReportCombinedViewInternal,
  api::{CommunityReportResponse, ResolveCommunityReport},
};
use lemmy_diesel_utils::utils::diesel_string_update;
use lemmy_utils::error::LemmyResult;

pub async fn resolve_community_report(
  Json(data): Json<ResolveCommunityReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person = &local_user_view.person;

  let conclusion = diesel_string_update(data.conclusion.as_deref());

  let form = UpdateCommunityReportForm {
    resolver_id: Some(person.id),
    resolved: Some(data.resolved),
    conclusion,
    updated_at: Some(Utc::now()),
  };

  CommunityReport::update_resolved(&mut context.pool(), report_id, &form).await?;

  let community_report_view =
    ReportCombinedViewInternal::read_community_report(&mut context.pool(), report_id, person)
      .await?;
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
