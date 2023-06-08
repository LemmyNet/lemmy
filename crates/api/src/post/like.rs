use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{CreatePostLike, PostResponse},
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    check_downvotes_enabled,
    local_user_view_from_jwt,
    mark_post_as_read,
  },
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    post::{Post, PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for CreatePostLike {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &CreatePostLike = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, &local_site)?;

    // Check for a community ban
    let post_id = data.post_id;
    let post = Post::read(context.pool(), post_id).await?;

    check_community_ban(local_user_view.person.id, post.community_id, context.pool()).await?;
    check_community_deleted_or_removed(post.community_id, context.pool()).await?;

    let like_form = PostLikeForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
      score: data.score,
    };

    // Remove any likes first
    let person_id = local_user_view.person.id;

    PostLike::remove(context.pool(), person_id, post_id).await?;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      PostLike::like(context.pool(), &like_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_post"))?;
    }

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    build_post_response(
      context,
      post.community_id,
      local_user_view.person.id,
      post_id,
    )
    .await
  }
}
