use activitypub_federation::config::Data;
use actix_web::web::Json;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{source::post_report::PostReport, traits::Reportable};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  api::{PostReportResponse, ResolvePostReport},
  ReportCombinedViewInternal,
};
use lemmy_utils::error::LemmyResult;

/// Resolves or unresolves a post report and notifies the moderators of the community
pub async fn resolve_post_report(
  data: Json<ResolvePostReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostReportResponse>> {
  let report_id = data.report_id;
  let person = &local_user_view.person;
  let report =
    ReportCombinedViewInternal::read_post_report(&mut context.pool(), report_id, person).await?;

  let person = &local_user_view.person;
  check_community_mod_action(
    &local_user_view,
    &report.community,
    true,
    &mut context.pool(),
  )
  .await?;

  PostReport::update_resolved(&mut context.pool(), report_id, person.id, data.resolved).await?;

  let post_report_view =
    ReportCombinedViewInternal::read_post_report(&mut context.pool(), report_id, person).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: post_report_view.post.ap_id.inner().clone(),
      actor: local_user_view.person,
      report_creator: report.creator,
      receiver: Either::Right(post_report_view.community.clone()),
    },
    &context,
  )?;

  Ok(Json(PostReportResponse { post_report_view }))
}
