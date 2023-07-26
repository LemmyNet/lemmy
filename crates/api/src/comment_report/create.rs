use crate::{check_report_reason, Perform};
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentReportResponse, CreateCommentReport},
  context::LemmyContext,
  utils::{
    check_community_ban,
    local_user_view_from_jwt,
    sanitize_html,
    send_new_report_email_to_admins,
  },
};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    local_site::LocalSite,
  },
  traits::Reportable,
};
use lemmy_db_views::structs::{CommentReportView, CommentView};
use lemmy_utils::error::LemmyError;

/// Creates a comment report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreateCommentReport {
  type Response = CommentReportResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<CommentReportResponse, LemmyError> {
    let data: &CreateCommentReport = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let reason = sanitize_html(self.reason.trim());
    check_report_reason(&reason, &local_site)?;

    let person_id = local_user_view.person.id;
    let comment_id = data.comment_id;
    let comment_view = CommentView::read(context.pool(), comment_id, None).await?;

    check_community_ban(person_id, comment_view.community.id, context.pool()).await?;

    let report_form = CommentReportForm {
      creator_id: person_id,
      comment_id,
      original_comment_text: comment_view.comment.content,
      reason,
    };

    let report = CommentReport::report(context.pool(), &report_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_report"))?;

    let comment_report_view = CommentReportView::read(context.pool(), report.id, person_id).await?;

    // Email the admins
    if local_site.reports_email_admins {
      send_new_report_email_to_admins(
        &comment_report_view.creator.name,
        &comment_report_view.comment_creator.name,
        context.pool(),
        context.settings(),
      )
      .await?;
    }

    Ok(CommentReportResponse {
      comment_report_view,
    })
  }
}
