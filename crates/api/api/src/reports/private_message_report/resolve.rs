use activitypub_federation::config::Data;
use actix_web::web::Json;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{person::Person, private_message_report::PrivateMessageReport, site::Site},
  traits::Reportable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  ReportCombinedViewInternal,
  api::{PrivateMessageReportResponse, ResolvePrivateMessageReport},
};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn resolve_pm_report(
  Json(data): Json<ResolvePrivateMessageReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person = &local_user_view.person;
  PrivateMessageReport::update_resolved(
    &mut context.pool(),
    report_id,
    person.id,
    data.resolved,
    data.resolve_reason,
  )
  .await?;

  let private_message_report_view =
    ReportCombinedViewInternal::read_private_message_report(&mut context.pool(), report_id, person)
      .await?;

  let recipient = Person::read(
    &mut context.pool(),
    private_message_report_view.private_message.recipient_id,
  )
  .await?;
  let site = Site::read_from_instance_id(&mut context.pool(), recipient.instance_id).await?;
  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: private_message_report_view
        .private_message
        .ap_id
        .inner()
        .clone(),
      actor: local_user_view.person,
      report_creator: private_message_report_view.creator.clone(),
      receiver: Either::Left(site),
    },
    &context,
  )?;

  Ok(Json(PrivateMessageReportResponse {
    private_message_report_view,
  }))
}
