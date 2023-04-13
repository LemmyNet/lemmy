use crate::{check_report_reason, Perform};
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{CreatePrivateMessageReport, PrivateMessageReportResponse},
  utils::{get_local_user_view_from_jwt, send_new_report_email_to_admins},
  websocket::UserOperation,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{
    local_site::LocalSite,
    private_message::PrivateMessage,
    private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views::structs::PrivateMessageReportView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for CreatePrivateMessageReport {
  type Response = PrivateMessageReportResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&self.auth, context.pool(), context.secret()).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let reason = self.reason.trim();
    check_report_reason(reason, &local_site)?;

    let person_id = local_user_view.person.id;
    let private_message_id = self.private_message_id;
    let private_message = PrivateMessage::read(context.pool(), private_message_id).await?;

    let report_form = PrivateMessageReportForm {
      creator_id: person_id,
      private_message_id,
      original_pm_text: private_message.content,
      reason: reason.to_owned(),
    };

    let report = PrivateMessageReport::report(context.pool(), &report_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_report"))?;

    let private_message_report_view =
      PrivateMessageReportView::read(context.pool(), report.id).await?;

    // Email the admins
    if local_site.reports_email_admins {
      send_new_report_email_to_admins(
        &private_message_report_view.creator.name,
        &private_message_report_view.private_message_creator.name,
        context.pool(),
        context.settings(),
      )
      .await?;
    }

    let res = PrivateMessageReportResponse {
      private_message_report_view,
    };

    context.send_mod_ws_message(
      &UserOperation::CreatePrivateMessageReport,
      &res,
      CommunityId(0),
      websocket_id,
    )?;

    // TODO: consider federating this

    Ok(res)
  }
}
