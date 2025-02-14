use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  comment::{ListCommentLikes, ListCommentLikesResponse},
  context::LemmyContext,
  utils::is_mod_or_admin,
};
use lemmy_db_schema::{
  source::person::Person,
  traits::{PageCursorBuilder, PageCursorReader},
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
    &local_user_view.person,
    comment_view.community.id,
  )
  .await?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(Person::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let comment_likes = VoteView::list_for_comment(
    &mut context.pool(),
    data.comment_id,
    cursor_data,
    data.page_back,
  )
  .await?;

  let next_page = comment_likes.last().map(PageCursorBuilder::cursor);

  Ok(Json(ListCommentLikesResponse {
    comment_likes,
    next_page,
  }))
}
