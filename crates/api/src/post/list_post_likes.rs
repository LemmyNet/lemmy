use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{ListPostLikes, ListPostLikesResponse},
  utils::is_mod_or_admin,
};
use lemmy_db_schema::{source::post::Post, traits::Crud};
use lemmy_db_views::structs::{LocalUserView, VoteView};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

/// Lists likes for a post
#[tracing::instrument(skip(context))]
pub async fn list_post_likes(
  data: Query<ListPostLikes>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPostLikesResponse>> {
  let post = Post::read(&mut context.pool(), data.post_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPost)?;
  is_mod_or_admin(
    &mut context.pool(),
    &local_user_view.person,
    post.community_id,
  )
  .await?;

  let post_likes =
    VoteView::list_for_post(&mut context.pool(), data.post_id, data.page, data.limit).await?;

  Ok(Json(ListPostLikesResponse { post_likes }))
}
