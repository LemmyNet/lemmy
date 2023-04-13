use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
  utils::{get_local_user_view_from_jwt, is_admin},
  websocket::UserOperation,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::private_message_report::PrivateMessageReport,
  traits::Reportable,
};
use lemmy_db_views::structs::PrivateMessageReportView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for ResolvePrivateMessageReport {
  type Response = PrivateMessageReportResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&self.auth, context.pool(), context.secret()).await?;

    is_admin(&local_user_view)?;

    let report_id = self.report_id;
    let person_id = local_user_view.person.id;
    if self.resolved {
      PrivateMessageReport::resolve(context.pool(), report_id, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_resolve_report"))?;
    } else {
      PrivateMessageReport::unresolve(context.pool(), report_id, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_resolve_report"))?;
    }

    let private_message_report_view =
      PrivateMessageReportView::read(context.pool(), report_id).await?;

    let res = PrivateMessageReportResponse {
      private_message_report_view,
    };

    context.send_mod_ws_message(
      &UserOperation::ResolvePrivateMessageReport,
      &res,
      CommunityId(0),
      websocket_id,
    )?;

    Ok(res)
  }
}
