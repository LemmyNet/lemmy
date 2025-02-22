use crate::check_report_reason;
use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  reports::community::{CreateCommunityReport, CommunityReportResponse},
  utils::send_new_report_email_to_admins,
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    community::Community,
    community_report::{CommunityReport, CommunityReportForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views::structs::{LocalUserView, CommunityReportView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

pub async fn create_community_report(
  data: Json<CreateCommunityReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityReportResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let reason = data.reason.trim().to_string();
  check_report_reason(&reason, &local_site)?;

  let person_id = local_user_view.person.id;
  let community_id = data.community_id;
  let community = Community::read(&mut context.pool(), community_id).await?;

  let report_form = CommunityReportForm {
    creator_id: person_id,
    community_id,
    original_community_banner: community.banner,
    original_community_description: community.description,
    original_community_icon: community.icon,
    original_community_name:community.name,
    original_community_sidebar:community.sidebar,
    original_community_title:community.title,
    reason,
  };

  let report = CommunityReport::report(&mut context.pool(), &report_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreateReport)?;

  let community_report_view =
    CommunityReportView::read(&mut context.pool(), report.id).await?;

  // Email the admins
  if local_site.reports_email_admins {
    send_new_report_email_to_admins(
      &community_report_view.creator.name,
      &community_report_view.community_creator.name,
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
