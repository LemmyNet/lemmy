use crate::check_report_reason;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use either::Either;
use lemmy_api_utils::{
  context::LemmyContext,
  plugins::plugin_hook_after,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    check_local_user_valid,
    check_post_deleted_or_removed,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_report_combined::{
  ReportCombinedViewInternal,
  api::{CreatePostReport, PostReportResponse},
};
use lemmy_db_views_site::SiteView;
use lemmy_email::admin::send_new_report_email_to_admins;
use lemmy_utils::error::LemmyResult;

/// Creates a post report and notifies the moderators of the community
pub async fn create_post_report(
  Json(data): Json<CreatePostReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostReportResponse>> {
  check_local_user_valid(&local_user_view)?;
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person = &local_user_view.person;
  let post_id = data.post_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_post = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;

  check_community_user_action(&local_user_view, &orig_post.community, &mut context.pool()).await?;

  check_post_deleted_or_removed(&orig_post.post)?;

  let report_form = PostReportForm {
    creator_id: person.id,
    post_id,
    original_post_name: orig_post.post.name,
    original_post_url: orig_post.post.url,
    original_post_body: orig_post.post.body,
    reason,
    violates_instance_rules: data.violates_instance_rules.unwrap_or_default(),
  };

  let report = PostReport::report(&mut context.pool(), &report_form).await?;

  let post_report_view =
    ReportCombinedViewInternal::read_post_report(&mut context.pool(), report.id, person).await?;
  plugin_hook_after("post_report_after_create", &post_report_view);

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
      object_id: orig_post.post.ap_id.inner().clone(),
      actor: local_user_view.person,
      receiver: Either::Right(orig_post.community),
      reason: data.reason.clone(),
    },
    &context,
  )?;

  Ok(Json(PostReportResponse { post_report_view }))
}
