use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  comment::{ListCommentLikes, ListCommentLikesResponse},
  context::LemmyContext,
  utils::is_mod_or_admin,
};
use lemmy_db_views::structs::{CommentView, LocalUserView, VoteView};
use lemmy_utils::error::LemmyResult;

/// Lists likes for a comment
pub async fn list_comment_likes(
  data: Query<ListCommentLikes>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListCommentLikesResponse>> {
  let comment_view = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
  )
  .await?;

  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view,
    comment_view.community.id,
  )
  .await?;

  let comment_likes =
    VoteView::list_for_comment(&mut context.pool(), data.comment_id, data.page, data.limit).await?;

  Ok(Json(ListCommentLikesResponse { comment_likes }))
}
