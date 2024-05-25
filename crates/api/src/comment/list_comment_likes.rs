use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  comment::{ListCommentLikes, ListCommentLikesResponse},
  context::LemmyContext,
  utils::is_mod_or_admin,
};
use lemmy_db_schema::viewer::Viewer;
use lemmy_db_views::structs::{CommentView, LocalUserView, VoteView};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

/// Lists likes for a comment
#[tracing::instrument(skip(context))]
pub async fn list_comment_likes(
  data: Query<ListCommentLikes>,
  context: Data<LemmyContext>,
  viewer: Viewer,
) -> LemmyResult<Json<ListCommentLikesResponse>> {
  let comment_view = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    viewer.person_id(),
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindComment)?;

  viewer.check_mod_or_admin(
    &mut context.pool(),
    comment_view.community.id,
  )
  .await?;

  let comment_likes =
    VoteView::list_for_comment(&mut context.pool(), data.comment_id, data.page, data.limit).await?;

  Ok(Json(ListCommentLikesResponse { comment_likes }))
}
