use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{DeletePost, PostResponse},
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    get_local_user_view_from_jwt,
  },
};
use lemmy_apub::activities::deletion::{send_apub_delete_in_community, DeletableObjects};
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};

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
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

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
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_deleted(conn, post_id, deleted)
    })
    .await??;

    let res = send_post_ws_message(
      data.post_id,
      UserOperationCrud::DeletePost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await?;

    // apub updates
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, orig_post.community_id)
    })
    .await??;
    let deletable = DeletableObjects::Post(Box::new(updated_post.into()));
    send_apub_delete_in_community(
      local_user_view.person,
      community,
      deletable,
      None,
      deleted,
      context,
    )
    .await?;
    Ok(res)
  }
}
