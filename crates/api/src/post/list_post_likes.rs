use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{ListPostLikes, ListPostLikesResponse},
  utils::is_mod_or_admin,
};
use lemmy_db_views::structs::{LocalUserView, PostView, VoteView};
use lemmy_utils::error::LemmyError;

/// Lists likes for a post
#[tracing::instrument(skip(context))]
pub async fn list_post_likes(
  data: Query<ListPostLikes>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ListPostLikesResponse>, LemmyError> {
  let post_view = PostView::read(
    &mut context.pool(),
    data.post_id,
    Some(local_user_view.person.id),
    false,
  )
  .await?;
  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view.person,
    post_view.community.id,
  )
  .await?;

  let post_likes =
    VoteView::list_for_post(&mut context.pool(), data.post_id, data.page, data.limit).await?;

  Ok(Json(ListPostLikesResponse { post_likes }))
}
