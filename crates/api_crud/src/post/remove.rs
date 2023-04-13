use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostResponse, RemovePost},
  utils::{check_community_ban, get_local_user_view_from_jwt, is_mod_or_admin},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::{
    moderator::{ModRemovePost, ModRemovePostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemovePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &RemovePost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(context.pool(), post_id).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;

    // Verify that only the mods can remove
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let removed = data.removed;
    Post::update(
      context.pool(),
      post_id,
      &PostUpdateForm::builder().removed(Some(removed)).build(),
    )
    .await?;

    // Mod tables
    let form = ModRemovePostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      removed: Some(removed),
      reason: data.reason.clone(),
    };
    ModRemovePost::create(context.pool(), &form).await?;

    let res = context
      .send_post_ws_message(
        &UserOperationCrud::RemovePost,
        data.post_id,
        websocket_id,
        Some(local_user_view.person.id),
      )
      .await?;

    Ok(res)
  }
}
