use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
  post::{
    CreatePostReport,
    ListPostReports,
    ListPostReportsResponse,
    PostReportResponse,
    ResolvePostReport,
  },
};
use lemmy_db_queries::Reportable;
use lemmy_db_schema::source::post_report::{PostReport, PostReportForm};
use lemmy_db_views::{
  post_report_view::{PostReportQueryBuilder, PostReportView},
  post_view::PostView,
};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};

/// Creates a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreatePostReport {
  type Response = PostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostReportResponse, LemmyError> {
    let data: &CreatePostReport = self;
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
    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, None)
    })
    .await??;

    check_community_ban(person_id, post_view.community.id, context.pool()).await?;

    let report_form = PostReportForm {
      creator_id: person_id,
      post_id,
      original_post_name: post_view.post.name,
      original_post_url: post_view.post.url,
      original_post_body: post_view.post.body,
      reason: data.reason.to_owned(),
    };

    let report = blocking(context.pool(), move |conn| {
      PostReport::report(conn, &report_form)
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_create_report"))?;

    let post_report_view = blocking(context.pool(), move |conn| {
      PostReportView::read(conn, report.id)
    })
    .await??;

    let res = PostReportResponse { post_report_view };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::CreatePostReport,
      response: res.clone(),
      community_id: post_view.community.id,
      websocket_id,
    });

    Ok(res)
  }
}

/// Resolves or unresolves a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for ResolvePostReport {
  type Response = PostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostReportResponse, LemmyError> {
    let data: &ResolvePostReport = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let report_id = data.report_id;
    let report = blocking(context.pool(), move |conn| {
      PostReportView::read(conn, report_id)
    })
    .await??;

    let person_id = local_user_view.person.id;
    is_mod_or_admin(context.pool(), person_id, report.community.id).await?;

    let resolved = data.resolved;
    let resolve_fun = move |conn: &'_ _| {
      if resolved {
        PostReport::resolve(conn, report_id, person_id)
      } else {
        PostReport::unresolve(conn, report_id, person_id)
      }
    };

    if blocking(context.pool(), resolve_fun).await?.is_err() {
      return Err(ApiError::err("couldnt_resolve_report").into());
    };

    let post_report_view = blocking(context.pool(), move |conn| {
      PostReportView::read(conn, report_id)
    })
    .await??;

    let res = PostReportResponse { post_report_view };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::ResolvePostReport,
      response: res.clone(),
      community_id: report.community.id,
      websocket_id,
    });

    Ok(res)
  }
}

/// Lists post reports for a community if an id is supplied
/// or returns all post reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListPostReports {
  type Response = ListPostReportsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListPostReportsResponse, LemmyError> {
    let data: &ListPostReports = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let community_id = data.community_id;

    let page = data.page;
    let limit = data.limit;
    let post_reports = blocking(context.pool(), move |conn| {
      PostReportQueryBuilder::create(conn, person_id)
        .community_id(community_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    let res = ListPostReportsResponse { post_reports };

    Ok(res)
  }
}
