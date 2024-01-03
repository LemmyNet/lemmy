use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  comment::{ListCommentLikes, ListCommentLikesResponse},
  context::LemmyContext,
  utils::is_admin,
};
use lemmy_db_views::structs::{LocalUserView, VoteView};
use lemmy_utils::error::LemmyError;

/// Lists likes for a comment
#[tracing::instrument(skip(context))]
pub async fn list_comment_likes(
  data: Query<ListCommentLikes>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ListCommentLikesResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let comment_likes =
    VoteView::list_for_comment(&mut context.pool(), data.comment_id, data.page, data.limit).await?;

  Ok(Json(ListCommentLikesResponse { comment_likes }))
}
