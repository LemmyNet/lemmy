use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostReportResponse, ResolvePostReport},
  utils::{get_local_user_view_from_jwt, is_mod_or_admin},
  websocket::UserOperation,
};
use lemmy_db_schema::{source::post_report::PostReport, traits::Reportable};
use lemmy_db_views::structs::PostReportView;
use lemmy_utils::{error::LemmyError, ConnectionId};

/// Resolves or unresolves a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for ResolvePostReport {
  type Response = PostReportResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostReportResponse, LemmyError> {
    let data: &ResolvePostReport = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let report_id = data.report_id;
    let person_id = local_user_view.person.id;
    let report = PostReportView::read(context.pool(), report_id, person_id).await?;

    let person_id = local_user_view.person.id;
    is_mod_or_admin(context.pool(), person_id, report.community.id).await?;

    if data.resolved {
      PostReport::resolve(context.pool(), report_id, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_resolve_report"))?;
    } else {
      PostReport::unresolve(context.pool(), report_id, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_resolve_report"))?;
    }

    let post_report_view = PostReportView::read(context.pool(), report_id, person_id).await?;

    let res = PostReportResponse { post_report_view };

    context.send_mod_ws_message(
      &UserOperation::ResolvePostReport,
      &res,
      report.community.id,
      websocket_id,
    )?;

    Ok(res)
  }
}
