use actix_web::web::Data;
use std::str::FromStr;

use lemmy_db::{comment_report::*, comment_view::*, post_report::*, post_view::*, Reportable, ReportType,};
use lemmy_structs::{blocking, report::*};
use lemmy_utils::{APIError, ConnectionId, LemmyError};
use lemmy_websocket::{LemmyContext, UserOperation, messages::SendUserRoomMessage};

use crate::{check_community_ban, get_user_from_jwt, is_mod_or_admin, Perform};

const MAX_REPORT_LEN: usize = 1000;

#[async_trait::async_trait(?Send)]
impl Perform for CreateReport {
  type Response = CreateReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CreateReportResponse, LemmyError> {
    let data: &CreateReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // check size of report and check for whitespace
    let reason = data.reason.clone();
    if reason.trim().is_empty() {
      return Err(APIError::err("report_reason_required").into());
    }
    if reason.len() > MAX_REPORT_LEN {
      return Err(APIError::err("report_too_long").into());
    }

    let report_type = ReportType::from_str(&data.report_type)?;
    let user_id = user.id;
    match report_type {
      ReportType::Comment => { create_comment_report(context, data, user_id).await?; }
      ReportType::Post => { create_post_report(context, data, user_id).await?; }
    }

    // to build on this, the user should get a success response, however
    // mods should get a different response with more details
    let res = CreateReportResponse { success: true };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::CreateReport,
      response: res.clone(),
      recipient_id: user.id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetReportCount {
  type Response = GetReportCountResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<GetReportCountResponse, LemmyError> {
    let data: &GetReportCount = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;
    let community_id = data.community;

    // Check for mod/admin privileges
    is_mod_or_admin(context.pool(), user.id, community_id).await?;

    let comment_reports = blocking(context.pool(), move |conn| {
      CommentReportQueryBuilder::create(conn)
        .community_id(community_id)
        .resolved(false)
        .count()
    })
    .await??;
    let post_reports = blocking(context.pool(), move |conn| {
      PostReportQueryBuilder::create(conn)
        .community_id(community_id)
        .resolved(false)
        .count()
    })
    .await??;

    let res = GetReportCountResponse {
      community: community_id,
      comment_reports,
      post_reports,
    };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::ListReports,
      response: res.clone(),
      recipient_id: user.id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ListReports {
  type Response = ListReportsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<ListReportsResponse, LemmyError> {
    let data: &ListReports = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;
    let community_id = data.community;

    // Check for mod/admin privileges
    is_mod_or_admin(context.pool(), user.id, community_id).await?;

    let page = data.page;
    let limit = data.limit;
    let comments = blocking(context.pool(), move |conn| {
      CommentReportQueryBuilder::create(conn)
        .community_id(community_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    let posts = blocking(context.pool(), move |conn| {
      PostReportQueryBuilder::create(conn)
          .community_id(community_id)
          .page(page)
          .limit(limit)
          .list()
    })
        .await??;

    let res = ListReportsResponse { comments, posts };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::ListReports,
      response: res.clone(),
      recipient_id: user.id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ResolveReport {
  type Response = ResolveReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<ResolveReportResponse, LemmyError> {
    let data: &ResolveReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let report_type = ReportType::from_str(&data.report_type)?;
    let user_id = user.id;
    match report_type {
      ReportType::Comment => { resolve_comment_report(context, data, user_id).await?; }
      ReportType::Post => { resolve_post_report(context, data, user_id).await?; }
    }

    let report_id = data.report_id;
    let res = ResolveReportResponse {
      report_type: data.report_type.to_owned(),
      report_id,
      resolved: true,
    };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::ResolveReport,
      response: res.clone(),
      recipient_id: user.id,
      websocket_id,
    });

    Ok(res)
  }
}

async fn create_comment_report(
  context: &Data<LemmyContext>,
  data: &CreateReport,
  user_id: i32,
) -> Result<(), LemmyError> {
  let comment_id = data.entity_id;
  let comment = blocking(context.pool(), move |conn| {
    CommentView::read(&conn, comment_id, None)
  }).await??;

  check_community_ban(user_id, comment.community_id, context.pool()).await?;

  let report_form = CommentReportForm {
    creator_id: user_id,
    comment_id,
    comment_text: comment.content,
    reason: data.reason.to_owned(),
  };

  return match blocking(context.pool(), move |conn| {
    CommentReport::report(conn, &report_form)
  }).await? {
    Ok(_) => Ok(()),
    Err(_e) => Err(APIError::err("couldnt_create_report").into())
  };
}

async fn create_post_report(
  context: &Data<LemmyContext>,
  data: &CreateReport,
  user_id: i32,
) -> Result<(), LemmyError> {
  let post_id = data.entity_id;
  let post = blocking(context.pool(), move |conn| {
    PostView::read(&conn, post_id, None)
  }).await??;

  check_community_ban(user_id, post.community_id, context.pool()).await?;

  let report_form = PostReportForm {
    creator_id: user_id,
    post_id,
    post_name: post.name,
    post_url: post.url,
    post_body: post.body,
    reason: data.reason.to_owned(),
  };

  return match blocking(context.pool(), move |conn| {
    PostReport::report(conn, &report_form)
  }).await? {
    Ok(_) => Ok(()),
    Err(_e) => Err(APIError::err("couldnt_create_report").into())
  };
}

async fn resolve_comment_report(
  context: &Data<LemmyContext>,
  data: &ResolveReport,
  user_id: i32,
) -> Result<(), LemmyError> {
  let report_id = data.report_id;
  let report = blocking(context.pool(), move |conn| {
    CommentReportView::read(&conn, report_id)
  }).await??;

  is_mod_or_admin(context.pool(), user_id, report.community_id).await?;

  let resolved = data.resolved;
  let resolve_fun = move |conn: &'_ _| {
    if resolved {
      CommentReport::resolve(conn, report_id.clone(), user_id)
    } else {
      CommentReport::unresolve(conn, report_id.clone())
    }
  };

  if blocking(context.pool(),resolve_fun).await?.is_err() {
    return Err(APIError::err("couldnt_resolve_report").into())
  };

  Ok(())
}

async fn resolve_post_report(
  context: &Data<LemmyContext>,
  data: &ResolveReport,
  user_id: i32,
) -> Result<(), LemmyError> {
  let report_id = data.report_id;
  let report = blocking(context.pool(), move |conn| {
    PostReportView::read(&conn, report_id)
  }).await??;

  is_mod_or_admin(context.pool(), user_id, report.community_id).await?;

  let resolved = data.resolved;
  let resolve_fun = move |conn: &'_ _| {
    if resolved {
      PostReport::resolve(conn, report_id.clone(), user_id)
    } else {
      PostReport::unresolve(conn, report_id.clone())
    }
  };

  if blocking(context.pool(),resolve_fun).await?.is_err() {
    return Err(APIError::err("couldnt_resolve_report").into())
  };

  Ok(())
}
