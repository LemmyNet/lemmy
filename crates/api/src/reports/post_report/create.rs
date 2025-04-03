use crate::check_report_reason;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  reports::post::{CreatePostReport, PostReportResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_user_action, check_post_deleted_or_removed, slur_regex},
};
use lemmy_db_schema::{
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
};
use lemmy_db_views::structs::{LocalUserView, PostReportView, PostView, SiteView};
use lemmy_email::admin::send_new_report_email_to_admins;
use lemmy_utils::error::LemmyResult;

/// Creates a post report and notifies the moderators of the community
pub async fn create_post_report(
  data: Json<CreatePostReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostReportResponse>> {
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person_id = local_user_view.person.id;
  let post_id = data.post_id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_view =
    PostView::read(&mut context.pool(), post_id, None, local_instance_id, false).await?;

  check_community_user_action(&local_user_view, &post_view.community, &mut context.pool()).await?;

  check_post_deleted_or_removed(&post_view.post)?;

  let report_form = PostReportForm {
    creator_id: person_id,
    post_id,
    original_post_name: post_view.post.name,
    original_post_url: post_view.post.url,
    original_post_body: post_view.post.body,
    reason,
    violates_instance_rules: data.violates_instance_rules.unwrap_or_default(),
  };

  let report = PostReport::report(&mut context.pool(), &report_form).await?;

  let post_report_view = PostReportView::read(&mut context.pool(), report.id, person_id).await?;

  // Email the admins
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  if local_site.reports_email_admins {
    send_new_report_email_to_admins(
      &post_report_view.creator.name,
      &post_report_view.post_creator.name,
      &mut context.pool(),
      context.settings(),
    )
    .await?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::CreateReport {
      object_id: post_view.post.ap_id.inner().clone(),
      actor: local_user_view.person,
      community: post_view.community,
      reason: data.reason.clone(),
    },
    &context,
  )?;

  Ok(Json(PostReportResponse { post_report_view }))
}
