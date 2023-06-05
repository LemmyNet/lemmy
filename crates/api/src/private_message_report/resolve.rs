use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{source::private_message_report::PrivateMessageReport, traits::Reportable};
use lemmy_db_views::structs::PrivateMessageReportView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for ResolvePrivateMessageReport {
  type Response = PrivateMessageReportResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&self.auth, context).await?;

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

    Ok(PrivateMessageReportResponse {
      private_message_report_view,
    })
  }
}
