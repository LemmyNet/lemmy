use actix_web::web::Data;

use lemmy_db::{
  comment_report::*,
  comment_view::*,
  community_view::*,
  post_report::*,
  post_view::*,
  Reportable,
  user_view::UserView,
};
use lemmy_structs::{blocking, report::*};
use lemmy_utils::{APIError, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

use crate::{check_community_ban, get_user_from_jwt, Perform};

const MAX_REPORT_LEN: usize = 1000;

#[async_trait::async_trait(?Send)]
impl Perform for CreateCommentReport {
  type Response = CommentReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentReportResponse, LemmyError> {
    let data: &CreateCommentReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;
    
    // Check size of report and check for whitespace
    let reason: Option<String> = match data.reason.clone() {
      Some(s) if s.trim().is_empty() => None,
      Some(s) if s.len() > MAX_REPORT_LEN => {
        return Err(APIError::err("report_too_long").into());
      }
      Some(s) => Some(s),
      None => None,
    };

    // Fetch comment information
    let comment_id = data.comment;
    let comment = blocking(context.pool(), move |conn| CommentView::read(&conn, comment_id, None)).await??;

    // Check for community ban
    check_community_ban(user.id, comment.community_id, context.pool()).await?;

    // Insert the report
    let comment_time = match comment.updated {
      Some(s) => s,
      None => comment.published,
    };
    let report_form = CommentReportForm {
      time: None, // column defaults to now() in table
      reason,
      resolved: None, // column defaults to false
      user_id: user.id,
      comment_id,
      comment_text: comment.content,
      comment_time,
    };
    blocking(context.pool(), move |conn| CommentReport::report(conn, &report_form)).await??;

    Ok(CommentReportResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for CreatePostReport {
  type Response = PostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PostReportResponse, LemmyError> {
    let data: &CreatePostReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Check size of report and check for whitespace
    let reason: Option<String> = match data.reason.clone() {
      Some(s) if s.trim().is_empty() => None,
      Some(s) if s.len() > MAX_REPORT_LEN => {
        return Err(APIError::err("report_too_long").into());
      }
      Some(s) => Some(s),
      None => None,
    };

    // Fetch post information from the database
    let post_id = data.post;
    let post = blocking(context.pool(), move |conn| PostView::read(&conn, post_id, None)).await??;

    // Check for community ban
    check_community_ban(user.id, post.community_id, context.pool()).await?;

    // Insert the report
    let post_time = match post.updated {
      Some(s) => s,
      None => post.published,
    };
    let report_form = PostReportForm {
      time: None, // column defaults to now() in table
      reason,
      resolved: None, // column defaults to false
      user_id: user.id,
      post_id,
      post_name: post.name,
      post_url: post.url,
      post_body: post.body,
      post_time,
    };
    blocking(context.pool(), move |conn| PostReport::report(conn, &report_form)).await??;

    Ok(PostReportResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetReportCount {
  type Response = GetReportCountResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetReportCountResponse, LemmyError> {
    let data: &GetReportCount = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community;
    //Check community exists.
    let community_id = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??
    .id;

    // Check community ban
    check_community_ban(user.id, data.community, context.pool()).await?;

    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user.id) {
      return Err(APIError::err("report_view_not_allowed").into());
    }

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

    let response = GetReportCountResponse {
      community: community_id,
      comment_reports,
      post_reports,
    };

    Ok(response)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ListCommentReports {
  type Response = ListCommentReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommentReportResponse, LemmyError> {
    let data: &ListCommentReports = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community;
    //Check community exists.
    let community_id = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??
    .id;

    check_community_ban(user.id, data.community, context.pool()).await?;

    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user.id) {
      return Err(APIError::err("report_view_not_allowed").into());
    }

    let page = data.page;
    let limit = data.limit;
    let reports = blocking(context.pool(), move |conn| {
      CommentReportQueryBuilder::create(conn)
        .community_id(community_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(ListCommentReportResponse { reports })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ListPostReports {
  type Response = ListPostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListPostReportResponse, LemmyError> {
    let data: &ListPostReports = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community;
    //Check community exists.
    let community_id = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??
    .id;
    // Check for community ban
    check_community_ban(user.id, data.community, context.pool()).await?;

    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user.id) {
      return Err(APIError::err("report_view_not_allowed").into());
    }

    let page = data.page;
    let limit = data.limit;
    let reports = blocking(context.pool(), move |conn| {
      PostReportQueryBuilder::create(conn)
        .community_id(community_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(ListPostReportResponse { reports })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ResolveCommentReport {
  type Response = ResolveCommentReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ResolveCommentReportResponse, LemmyError> {
    let data: &ResolveCommentReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Fetch the report view
    let report_id = data.report;
    let report = blocking(context.pool(), move |conn| CommentReportView::read(&conn, &report_id)).await??;

    // Check for community ban
    check_community_ban(user.id, report.community_id, context.pool()).await?;

    // Check for mod/admin privileges
    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        CommunityModeratorView::for_community(conn, report.community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user.id) {
      return Err(APIError::err("resolve_report_not_allowed").into());
    }

    blocking(context.pool(), move |conn| {
      CommentReport::resolve(conn, &report_id.clone())
    })
    .await??;

    Ok(ResolveCommentReportResponse {
      report: report_id,
      resolved: true,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ResolvePostReport {
  type Response = ResolvePostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ResolvePostReportResponse, LemmyError> {
    let data: &ResolvePostReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Fetch the report view
    let report_id = data.report;
    let report = blocking(context.pool(), move |conn| PostReportView::read(&conn, &report_id)).await??;

    // Check for community ban
    check_community_ban(user.id, report.community_id, context.pool()).await?;

    // Check for mod/admin privileges
    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        CommunityModeratorView::for_community(conn, report.community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(context.pool(), move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user.id) {
      return Err(APIError::err("resolve_report_not_allowed").into());
    }

    blocking(context.pool(), move |conn| {
      PostReport::resolve(conn, &report_id.clone())
    })
    .await??;

    Ok(ResolvePostReportResponse {
      report: report_id,
      resolved: true,
    })
  }
}
