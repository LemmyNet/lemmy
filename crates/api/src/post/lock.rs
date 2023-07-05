use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{LockPost, PostResponse},
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    is_mod_or_admin,
    local_user_view_from_jwt,
  },
};
use lemmy_db_schema::{
  source::{
    moderator::{ModLockPost, ModLockPostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for LockPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &LockPost = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(context.pool(), post_id).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can lock
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let locked = data.locked;
    Post::update(
      context.pool(),
      post_id,
      &PostUpdateForm::builder().locked(Some(locked)).build(),
    )
    .await?;

    // Mod tables
    let form = ModLockPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      locked: Some(locked),
    };
    ModLockPost::create(context.pool(), &form).await?;

    build_post_response(
      context,
      orig_post.community_id,
      local_user_view.person.id,
      post_id,
    )
    .await
  }
}
