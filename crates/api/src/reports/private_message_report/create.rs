use crate::check_report_reason;
use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  reports::private_message::{CreatePrivateMessageReport, PrivateMessageReportResponse},
  utils::slur_regex,
};
use lemmy_db_schema::{
  source::{
    private_message::PrivateMessage,
    private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_reports::PrivateMessageReportView;
use lemmy_db_views_site::SiteView;
use lemmy_email::admin::send_new_report_email_to_admins;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn create_pm_report(
  data: Json<CreatePrivateMessageReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageReportResponse>> {
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person_id = local_user_view.person.id;
  let private_message_id = data.private_message_id;
  let private_message = PrivateMessage::read(&mut context.pool(), private_message_id).await?;

  // Make sure that only the recipient of the private message can create a report
  if person_id != private_message.recipient_id {
    Err(LemmyErrorType::CouldntCreateReport)?
  }

  let report_form = PrivateMessageReportForm {
    creator_id: person_id,
    private_message_id,
    original_pm_text: private_message.content,
    reason,
  };

  let report = PrivateMessageReport::report(&mut context.pool(), &report_form).await?;

  let private_message_report_view =
    PrivateMessageReportView::read(&mut context.pool(), report.id).await?;

  // Email the admins
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  if local_site.reports_email_admins {
    send_new_report_email_to_admins(
      &private_message_report_view.creator.name,
      &private_message_report_view.private_message_creator.name,
      &mut context.pool(),
      context.settings(),
    )
    .await?;
  }

  // TODO: consider federating this

  Ok(Json(PrivateMessageReportResponse {
    private_message_report_view,
  }))
}
