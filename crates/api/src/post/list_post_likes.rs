use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{ListPostLikes, ListPostLikesResponse},
  utils::is_admin,
};
use lemmy_db_views::structs::{LocalUserView, VoteView};
use lemmy_utils::error::LemmyError;

/// Lists likes for a post
#[tracing::instrument(skip(context))]
pub async fn list_post_likes(
  data: Query<ListPostLikes>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ListPostLikesResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let post_likes =
    VoteView::list_for_post(&mut context.pool(), data.post_id, data.page, data.limit).await?;

  Ok(Json(ListPostLikesResponse { post_likes }))
}
