use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{DeletePost, PostResponse},
  utils::{check_community_ban, check_community_deleted_or_removed, get_local_user_view_from_jwt},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::post::{Post, PostUpdateForm},
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeletePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &DeletePost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(context.pool(), post_id).await?;

    // Dont delete it if its already been deleted.
    if orig_post.deleted == data.deleted {
      return Err(LemmyError::from_message("couldnt_update_post"));
    }

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the creator can delete
    if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
      return Err(LemmyError::from_message("no_post_edit_allowed"));
    }

    // Update the post
    let post_id = data.post_id;
    let deleted = data.deleted;
    Post::update(
      context.pool(),
      post_id,
      &PostUpdateForm::builder().deleted(Some(deleted)).build(),
    )
    .await?;

    let res = context
      .send_post_ws_message(
        &UserOperationCrud::DeletePost,
        data.post_id,
        websocket_id,
        Some(local_user_view.person.id),
      )
      .await?;

    Ok(res)
  }
}
