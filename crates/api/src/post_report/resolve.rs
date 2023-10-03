use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostReportResponse, ResolvePostReport},
  utils::is_mod_or_admin,
};
use lemmy_db_schema::{source::post_report::PostReport, traits::Reportable};
use lemmy_db_views::structs::{LocalUserView, PostReportView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

/// Resolves or unresolves a post report and notifies the moderators of the community
#[tracing::instrument(skip(context))]
pub async fn resolve_post_report(
  data: Json<ResolvePostReport>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PostReportResponse>, LemmyError> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;
  is_mod_or_admin(&mut context.pool(), person_id, report.community.id).await?;

  if data.resolved {
    PostReport::resolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  } else {
    PostReport::unresolve(&mut context.pool(), report_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)?;
  }

  let post_report_view = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  Ok(Json(PostReportResponse { post_report_view }))
}
