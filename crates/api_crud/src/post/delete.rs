use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  check_community_ban,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
  post::*,
};
use lemmy_apub::activities::deletion::{send_apub_delete, send_apub_remove};
use lemmy_db_queries::{source::post::Post_, Crud};
use lemmy_db_schema::source::{community::Community, moderator::*, post::*};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeletePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &DeletePost = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&&context.pool.get().await?, post_id)?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can delete
    if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
      return Err(ApiError::err("no_post_edit_allowed").into());
    }

    // Update the post
    let post_id = data.post_id;
    let deleted = data.deleted;
    let updated_post = Post::update_deleted(&&context.pool.get().await?, post_id, deleted)?;

    // apub updates
    let community = Community::read(&&context.pool.get().await?, orig_post.community_id)?;
    send_apub_delete(
      &local_user_view.person,
      &community,
      updated_post.ap_id.into(),
      deleted,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperationCrud::DeletePost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemovePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &RemovePost = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&&context.pool.get().await?, post_id)?;

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
    let updated_post = Post::update_removed(&&context.pool.get().await?, post_id, removed)?;

    // Mod tables
    let form = ModRemovePostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
    };
    ModRemovePost::create(&&context.pool.get().await?, &form)?;

    // apub updates
    let community = Community::read(&&context.pool.get().await?, orig_post.community_id)?;
    send_apub_remove(
      &local_user_view.person,
      &community,
      updated_post.ap_id.into(),
      data.reason.clone().unwrap_or_else(|| "".to_string()),
      removed,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperationCrud::RemovePost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
