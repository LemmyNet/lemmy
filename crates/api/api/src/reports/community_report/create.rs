use crate::check_report_reason;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::slur_regex,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    community_report::{CommunityReport, CommunityReportForm},
    site::Site,
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_reports::{
  api::{CommunityReportResponse, CreateCommunityReport},
  CommunityReportView,
};
use lemmy_db_views_site::SiteView;
use lemmy_email::admin::send_new_report_email_to_admins;
use lemmy_utils::error::LemmyResult;

pub async fn create_community_report(
  data: Json<CreateCommunityReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityReportResponse>> {
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person_id = local_user_view.person.id;
  let community_id = data.community_id;
  let community = Community::read(&mut context.pool(), community_id).await?;
  let site = Site::read_from_instance_id(&mut context.pool(), community.instance_id).await?;

  let report_form = CommunityReportForm {
    creator_id: person_id,
    community_id,
    original_community_banner: community.banner,
    original_community_description: community.description,
    original_community_icon: community.icon,
    original_community_name: community.name,
    original_community_sidebar: community.sidebar,
    original_community_title: community.title,
    reason,
  };

  let report = CommunityReport::report(&mut context.pool(), &report_form).await?;

  let community_report_view =
    CommunityReportView::read(&mut context.pool(), report.id, person_id).await?;

  // Email the admins
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  if local_site.reports_email_admins {
    send_new_report_email_to_admins(
      &community_report_view.creator.name,
      // The argument here is normally the reported content's creator, but a community doesn't have
      // a single person to be considered the creator or the person responsible for the bad thing,
      // so the community name is used instead
      &community_report_view.community.name,
      &mut context.pool(),
      context.settings(),
    )
    .await?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::CreateReport {
      object_id: community.ap_id.inner().clone(),
      actor: local_user_view.person,
      receiver: Either::Left(site),
      reason: data.reason.clone(),
    },
    &context,
  )?;

  Ok(Json(CommunityReportResponse {
    community_report_view,
  }))
}
