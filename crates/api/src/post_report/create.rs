use crate::check_report_reason;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  post::{CreatePostReport, PostReportResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    check_post_deleted_or_removed,
    send_new_report_email_to_admins,
  },
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_db_views::structs::{LocalUserView, PostReportView, PostView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

/// Creates a post report and notifies the moderators of the community
#[tracing::instrument(skip(context))]
pub async fn create_post_report(
  data: Json<CreatePostReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostReportResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let reason = data.reason.trim().to_string();
  check_report_reason(&reason, &local_site)?;

  let person_id = local_user_view.person.id;
  let post_id = data.post_id;
  let post_view = PostView::read(&mut context.pool(), post_id, None, false).await?;

  check_community_user_action(
    &local_user_view.person,
    post_view.community.id,
    &mut context.pool(),
  )
  .await?;

  check_post_deleted_or_removed(&post_view.post)?;

  let report_form = PostReportForm {
    creator_id: person_id,
    post_id,
    original_post_name: post_view.post.name,
    original_post_url: post_view.post.url,
    original_post_body: post_view.post.body,
    reason,
  };

  let report = PostReport::report(&mut context.pool(), &report_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreateReport)?;

  let post_report_view = PostReportView::read(&mut context.pool(), report.id, person_id).await?;

  // Email the admins
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
  )
  .await?;

  Ok(Json(PostReportResponse { post_report_view }))
}
