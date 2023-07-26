use crate::{check_report_reason, Perform};
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{CreatePostReport, PostReportResponse},
  utils::{
    check_community_ban,
    local_user_view_from_jwt,
    sanitize_html,
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
use lemmy_db_views::structs::{PostReportView, PostView};
use lemmy_utils::error::LemmyError;

/// Creates a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreatePostReport {
  type Response = PostReportResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostReportResponse, LemmyError> {
    let data: &CreatePostReport = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let reason = sanitize_html(self.reason.trim());
    check_report_reason(&reason, &local_site)?;

    let person_id = local_user_view.person.id;
    let post_id = data.post_id;
    let post_view = PostView::read(context.pool(), post_id, None, None).await?;

    check_community_ban(person_id, post_view.community.id, context.pool()).await?;

    let report_form = PostReportForm {
      creator_id: person_id,
      post_id,
      original_post_name: post_view.post.name,
      original_post_url: post_view.post.url,
      original_post_body: post_view.post.body,
      reason,
    };

    let report = PostReport::report(context.pool(), &report_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_report"))?;

    let post_report_view = PostReportView::read(context.pool(), report.id, person_id).await?;

    // Email the admins
    if local_site.reports_email_admins {
      send_new_report_email_to_admins(
        &post_report_view.creator.name,
        &post_report_view.post_creator.name,
        context.pool(),
        context.settings(),
      )
      .await?;
    }

    Ok(PostReportResponse { post_report_view })
  }
}
