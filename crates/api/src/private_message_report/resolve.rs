use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  private_message::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::private_message_report::PrivateMessageReport,
  traits::Reportable,
};
use lemmy_db_views::structs::PrivateMessageReportView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};

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

    let resolved = self.resolved;
    let report_id = self.report_id;
    let person_id = local_user_view.person.id;
    let resolve_fn = move |conn: &mut _| {
      if resolved {
        PrivateMessageReport::resolve(conn, report_id, person_id)
      } else {
        PrivateMessageReport::unresolve(conn, report_id, person_id)
      }
    };

    blocking(context.pool(), resolve_fn)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_resolve_report"))?;

    let private_message_report_view = blocking(context.pool(), move |conn| {
      PrivateMessageReportView::read(conn, report_id)
    })
    .await??;

    let res = PrivateMessageReportResponse {
      private_message_report_view,
    };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::ResolvePrivateMessageReport,
      response: res.clone(),
      community_id: CommunityId(0),
      websocket_id,
    });

    Ok(res)
  }
}
