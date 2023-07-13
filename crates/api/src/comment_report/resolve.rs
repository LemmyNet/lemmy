use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentReportResponse, ResolveCommentReport},
  context::LemmyContext,
  utils::{is_mod_or_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{source::comment_report::CommentReport, traits::Reportable};
use lemmy_db_views::structs::CommentReportView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

/// Resolves or unresolves a comment report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for ResolveCommentReport {
  type Response = CommentReportResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<CommentReportResponse, LemmyError> {
    let data: &ResolveCommentReport = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let report_id = data.report_id;
    let person_id = local_user_view.person.id;
    let report = CommentReportView::read(&mut context.pool(), report_id, person_id).await?;

    let person_id = local_user_view.person.id;
    is_mod_or_admin(&mut context.pool(), person_id, report.community.id).await?;

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

    Ok(CommentReportResponse {
      comment_report_view,
    })
  }
}
