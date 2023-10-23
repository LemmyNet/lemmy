use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetReportCount, GetReportCountResponse},
  utils::check_community_mod_action_opt,
};
use lemmy_db_views::structs::{
  CommentReportView,
  LocalUserView,
  PostReportView,
  PrivateMessageReportView,
};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn report_count(
  data: Json<GetReportCount>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<GetReportCountResponse>, LemmyError> {
  let person_id = local_user_view.person.id;
  let admin = local_user_view.local_user.admin;
  let community_id = data.community_id;

  check_community_mod_action_opt(&local_user_view, community_id, &mut context.pool()).await?;

  let comment_reports =
    CommentReportView::get_report_count(&mut context.pool(), person_id, admin, community_id)
      .await?;

  let post_reports =
    PostReportView::get_report_count(&mut context.pool(), person_id, admin, community_id).await?;

  let private_message_reports = if admin && community_id.is_none() {
    Some(PrivateMessageReportView::get_report_count(&mut context.pool()).await?)
  } else {
    None
  };

  Ok(Json(GetReportCountResponse {
    community_id,
    comment_reports,
    post_reports,
    private_message_reports,
  }))
}
