use actix_web::web::{Data, Json};
use lemmy_api_common::{
  comment::{CommentReportResponse, ResolveCommentReport},
  context::LemmyContext,
  utils::check_community_mod_action,
};
use lemmy_db_schema::{source::comment_report::CommentReport, traits::Reportable};
use lemmy_db_views::structs::{CommentReportView, LocalUserView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

/// Resolves or unresolves a comment report and notifies the moderators of the community
#[tracing::instrument(skip(context))]
pub async fn resolve_comment_report(
  data: Json<ResolveCommentReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<CommentReportResponse>, LemmyError> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = CommentReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;
  check_community_mod_action(
    &local_user_view.person,
    report.community.id,
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

  Ok(Json(CommentReportResponse {
    comment_report_view,
  }))
}
