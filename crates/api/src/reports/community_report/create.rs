use crate::check_report_reason;
use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  reports::community::{CommunityReportResponse, CreateCommunityReport},
  utils::slur_regex,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    community_report::{CommunityReport, CommunityReportForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views::structs::{CommunityReportView, LocalUserView, SiteView};
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

  // TODO: consider federating this

  Ok(Json(CommunityReportResponse {
    community_report_view,
  }))
}
