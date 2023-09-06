use crate::check_report_reason;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  comment::{CommentReportResponse, CreateCommentReport},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_ban, sanitize_html, send_new_report_email_to_admins},
};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    local_site::LocalSite,
  },
  traits::Reportable,
};
use lemmy_db_views::structs::{CommentReportView, CommentView, LocalUserView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

/// Creates a comment report and notifies the moderators of the community
#[tracing::instrument(skip(context))]
pub async fn create_comment_report(
  data: Json<CreateCommentReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<CommentReportResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let reason = sanitize_html(data.reason.trim());
  check_report_reason(&reason, &local_site)?;

  let person_id = local_user_view.person.id;
  let comment_id = data.comment_id;
  let comment_view = CommentView::read(&mut context.pool(), comment_id, None).await?;

  check_community_ban(person_id, comment_view.community.id, &mut context.pool()).await?;

  let report_form = CommentReportForm {
    creator_id: person_id,
    comment_id,
    original_comment_text: comment_view.comment.content,
    reason,
  };

  let report = CommentReport::report(&mut context.pool(), &report_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreateReport)?;

  let comment_report_view =
    CommentReportView::read(&mut context.pool(), report.id, person_id).await?;

  // Email the admins
  if local_site.reports_email_admins {
    send_new_report_email_to_admins(
      &comment_report_view.creator.name,
      &comment_report_view.comment_creator.name,
      &mut context.pool(),
      context.settings(),
    )
    .await?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::CreateReport(
      comment_view.comment.ap_id.inner().clone(),
      local_user_view.person,
      comment_view.community,
      data.reason.clone(),
    ),
    &context,
  )
  .await?;

  Ok(Json(CommentReportResponse {
    comment_report_view,
  }))
}
