use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  reports::post::{PostReportResponse, ResolvePostReport},
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{source::post_report::PostReport, traits::Reportable};
use lemmy_db_views::structs::{LocalUserView, PostReportView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

/// Resolves or unresolves a post report and notifies the moderators of the community
pub async fn resolve_post_report(
  data: Json<ResolvePostReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostReportResponse>> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;
  check_community_mod_action(
    &local_user_view.person,
    &report.community,
    true,
    &mut context.pool(),
  )
  .await?;

  if data.resolved {
    PostReport::resolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  } else {
    // TODO: not federated
    PostReport::unresolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  }

  let post_report_view = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      object_id: post_report_view.post.ap_id.inner().clone(),
      actor: local_user_view.person,
      report_creator: report.creator,
      community: post_report_view.community.clone(),
    },
    &context,
  )?;

  Ok(Json(PostReportResponse { post_report_view }))
}
