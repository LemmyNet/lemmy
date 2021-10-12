use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  comment::*,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
};
use lemmy_db_queries::Reportable;
use lemmy_db_schema::source::comment_report::*;
use lemmy_db_views::{
  comment_report_view::{CommentReportQueryBuilder, CommentReportView},
  comment_view::CommentView,
};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};

/// Creates a comment report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreateCommentReport {
  type Response = CommentReportResponse;

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
      return Err(ApiError::err("report_reason_required").into());
    }
    if reason.chars().count() > 1000 {
      return Err(ApiError::err("report_too_long").into());
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
    .map_err(|_| ApiError::err("couldnt_create_report"))?;

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

    Ok(res)
  }
}

/// Resolves or unresolves a comment report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for ResolveCommentReport {
  type Response = CommentReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentReportResponse, LemmyError> {
    let data: &ResolveCommentReport = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let report_id = data.report_id;
    let person_id = local_user_view.person.id;
    let report = blocking(context.pool(), move |conn| {
      CommentReportView::read(conn, report_id, person_id)
    })
    .await??;

    let person_id = local_user_view.person.id;
    is_mod_or_admin(context.pool(), person_id, report.community.id).await?;

    let resolved = data.resolved;
    let resolve_fun = move |conn: &'_ _| {
      if resolved {
        CommentReport::resolve(conn, report_id, person_id)
      } else {
        CommentReport::unresolve(conn, report_id, person_id)
      }
    };

    if blocking(context.pool(), resolve_fun).await?.is_err() {
      return Err(ApiError::err("couldnt_resolve_report").into());
    };

    let report_id = data.report_id;
    let comment_report_view = blocking(context.pool(), move |conn| {
      CommentReportView::read(conn, report_id, person_id)
    })
    .await??;

    let res = CommentReportResponse {
      comment_report_view,
    };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::ResolveCommentReport,
      response: res.clone(),
      community_id: report.community.id,
      websocket_id,
    });

    Ok(res)
  }
}

/// Lists comment reports for a community if an id is supplied
/// or returns all comment reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListCommentReports {
  type Response = ListCommentReportsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommentReportsResponse, LemmyError> {
    let data: &ListCommentReports = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let admin = local_user_view.person.admin;
    let community_id = data.community_id;
    let unresolved_only = data.unresolved_only;

    let page = data.page;
    let limit = data.limit;
    let comment_reports = blocking(context.pool(), move |conn| {
      CommentReportQueryBuilder::create(conn, person_id, admin)
        .community_id(community_id)
        .unresolved_only(unresolved_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    let res = ListCommentReportsResponse { comment_reports };

    Ok(res)
  }
}
