use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::is_mod_or_admin};
use lemmy_db_views_comment::{CommentView, api::ListCommentLikes};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_vote::VoteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

/// Lists likes for a comment
pub async fn list_comment_likes(
  Query(data): Query<ListCommentLikes>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<VoteView>>> {
  let local_instance_id = local_user_view.person.instance_id;

  let comment_view = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view,
    comment_view.community.id,
  )
  .await?;

  let comment_likes = VoteView::list_for_comment(
    &mut context.pool(),
    data.comment_id,
    data.page_cursor,
    data.limit,
    local_instance_id,
  )
  .await?;

  Ok(Json(comment_likes))
}
