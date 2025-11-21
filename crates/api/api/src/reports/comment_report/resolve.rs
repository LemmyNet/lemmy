use activitypub_federation::config::Data;
use actix_web::web::Json;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{source::comment_report::CommentReport, traits::Reportable};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  ReportCombinedViewInternal,
  api::{CommentReportResponse, ResolveCommentReport},
};
use lemmy_utils::error::LemmyResult;

/// Resolves or unresolves a comment report and notifies the moderators of the community
pub async fn resolve_comment_report(
  Json(data): Json<ResolveCommentReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentReportResponse>> {
  let report_id = data.report_id;
  let person = &local_user_view.person;
  let report =
    ReportCombinedViewInternal::read_comment_report(&mut context.pool(), report_id, person).await?;

  let person_id = local_user_view.person.id;
  check_community_mod_action(
    &local_user_view,
    &report.community,
    true,
    &mut context.pool(),
  )
  .await?;

  CommentReport::update_resolved(&mut context.pool(), report_id, person_id, data.resolved).await?;

  let report_id = data.report_id;
  let comment_report_view =
    ReportCombinedViewInternal::read_comment_report(&mut context.pool(), report_id, person).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: comment_report_view.comment.ap_id.inner().clone(),
      actor: local_user_view.person,
      report_creator: report.creator,
      receiver: Either::Right(comment_report_view.community.clone()),
    },
    &context,
  )?;

  Ok(Json(CommentReportResponse {
    comment_report_view,
  }))
}
