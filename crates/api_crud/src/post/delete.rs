use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{DeletePost, PostResponse},
  utils::{check_community_ban, check_community_deleted_or_removed, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::post::{Post, PostUpdateForm},
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeletePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &DeletePost = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&mut context.pool(), post_id).await?;

    // Dont delete it if its already been deleted.
    if orig_post.deleted == data.deleted {
      return Err(LemmyErrorType::CouldntUpdatePost)?;
    }

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      &mut context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, &mut context.pool()).await?;

    // Verify that only the creator can delete
    if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
      return Err(LemmyErrorType::NoPostEditAllowed)?;
    }

    // Update the post
    let post_id = data.post_id;
    let deleted = data.deleted;
    Post::update(
      &mut context.pool(),
      post_id,
      &PostUpdateForm::builder().deleted(Some(deleted)).build(),
    )
    .await?;

    build_post_response(
      context,
      orig_post.community_id,
      local_user_view.person.id,
      post_id,
    )
    .await
  }
}
