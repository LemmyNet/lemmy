use crate::{check_report_reason, Perform};
use activitypub_federation::core::object_id::ObjectId;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{CreatePostReport, PostReportResponse},
  utils::{blocking, check_community_ban, get_local_user_view_from_jwt},
};
use lemmy_apub::protocol::activities::community::report::Report;
use lemmy_db_schema::{
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
};
use lemmy_db_views::structs::{PostReportView, PostView};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};

/// Creates a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreatePostReport {
  type Response = PostReportResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostReportResponse, LemmyError> {
    let data: &CreatePostReport = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let reason = self.reason.trim();
    check_report_reason(reason, context)?;

    let person_id = local_user_view.person.id;
    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, None)
    })
    .await??;

    check_community_ban(person_id, post_view.community.id, context.pool()).await?;

    let report_form = PostReportForm {
      creator_id: person_id,
      post_id,
      original_post_name: post_view.post.name,
      original_post_url: post_view.post.url,
      original_post_body: post_view.post.body,
      reason: reason.to_owned(),
    };

    let report = blocking(context.pool(), move |conn| {
      PostReport::report(conn, &report_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_report"))?;

    let post_report_view = blocking(context.pool(), move |conn| {
      PostReportView::read(conn, report.id, person_id)
    })
    .await??;

    let res = PostReportResponse { post_report_view };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::CreatePostReport,
      response: res.clone(),
      community_id: post_view.community.id,
      websocket_id,
    });

    Report::send(
      ObjectId::new(post_view.post.ap_id),
      &local_user_view.person.into(),
      ObjectId::new(post_view.community.actor_id),
      reason.to_string(),
      context,
    )
    .await?;

    Ok(res)
  }
}
