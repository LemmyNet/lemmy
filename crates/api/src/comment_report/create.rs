use crate::Perform;
use activitypub_federation::core::object_id::ObjectId;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentReportResponse, CreateCommentReport},
  utils::{blocking, check_community_ban, get_local_user_view_from_jwt},
};
use lemmy_apub::protocol::activities::community::report::Report;
use lemmy_db_schema::{
  source::comment_report::{CommentReport, CommentReportForm},
  traits::Reportable,
};
use lemmy_db_views::structs::{CommentReportView, CommentView};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};

/// Creates a comment report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreateCommentReport {
  type Response = CommentReportResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentReportResponse, LemmyError> {
    let data: &CreateCommentReport = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // check size of report and check for whitespace
    let reason = data.reason.trim();
    if reason.is_empty() {
      return Err(LemmyError::from_message("report_reason_required"));
    }
    if reason.chars().count() > 1000 {
      return Err(LemmyError::from_message("report_too_long"));
    }

    let person_id = local_user_view.person.id;
    let comment_id = data.comment_id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    check_community_ban(person_id, comment_view.community.id, context.pool()).await?;

    let report_form = CommentReportForm {
      creator_id: person_id,
      comment_id,
      original_comment_text: comment_view.comment.content,
      reason: data.reason.to_owned(),
    };

    let report = blocking(context.pool(), move |conn| {
      CommentReport::report(conn, &report_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_report"))?;

    let comment_report_view = blocking(context.pool(), move |conn| {
      CommentReportView::read(conn, report.id, person_id)
    })
    .await??;

    let res = CommentReportResponse {
      comment_report_view,
    };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::CreateCommentReport,
      response: res.clone(),
      community_id: comment_view.community.id,
      websocket_id,
    });

    Report::send(
      ObjectId::new(comment_view.comment.ap_id),
      &local_user_view.person.into(),
      ObjectId::new(comment_view.community.actor_id),
      reason.to_string(),
      context,
    )
    .await?;

    Ok(res)
  }
}
