use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  reports::comment::{CommentReportResponse, ResolveCommentReport},
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{source::comment_report::CommentReport, traits::Reportable};
use lemmy_db_views::structs::{CommentReportView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

/// Resolves or unresolves a comment report and notifies the moderators of the community
pub async fn resolve_comment_report(
  data: Json<ResolveCommentReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentReportResponse>> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = CommentReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;
  check_community_mod_action(
    &local_user_view.person,
    &report.community,
    true,
    &mut context.pool(),
  )
  .await?;

  if data.resolved {
    CommentReport::resolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  } else {
    CommentReport::unresolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  }

  let report_id = data.report_id;
  let comment_report_view =
    CommentReportView::read(&mut context.pool(), report_id, person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: comment_report_view.comment.ap_id.inner().clone(),
      actor: local_user_view.person,
      report_creator: report.creator,
      community: comment_report_view.community.clone(),
    },
    &context,
  )?;

  Ok(Json(CommentReportResponse {
    comment_report_view,
  }))
}
