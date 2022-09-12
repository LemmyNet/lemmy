use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{GetReportCount, GetReportCountResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_views::structs::{CommentReportView, PostReportView, PrivateMessageReportView};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetReportCount {
  type Response = GetReportCountResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetReportCountResponse, LemmyError> {
    let data: &GetReportCount = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.person.id;
    let admin = local_user_view.person.admin;
    let community_id = data.community_id;

    let comment_reports = blocking(context.pool(), move |conn| {
      CommentReportView::get_report_count(conn, person_id, admin, community_id)
    })
    .await??;

    let post_reports = blocking(context.pool(), move |conn| {
      PostReportView::get_report_count(conn, person_id, admin, community_id)
    })
    .await??;

    let private_message_reports = if admin && community_id.is_none() {
      Some(
        blocking(context.pool(), move |conn| {
          PrivateMessageReportView::get_report_count(conn)
        })
        .await??,
      )
    } else {
      None
    };

    let res = GetReportCountResponse {
      community_id,
      comment_reports,
      post_reports,
      private_message_reports,
    };

    Ok(res)
  }
}
