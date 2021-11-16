use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  comment::*,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
};
use lemmy_apub::activities::deletion::{send_apub_delete, send_apub_remove, DeletableObjects};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    community::Community,
    moderator::{ModRemoveComment, ModRemoveCommentForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::comment_view::CommentView;
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{
  send::{send_comment_ws_message, send_local_notifs},
  LemmyContext,
  UserOperationCrud,
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteComment {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &DeleteComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    // Dont delete it if its already been deleted.
    if orig_comment.comment.deleted == data.deleted {
      return Err(ApiError::err_plain("couldnt_update_comment").into());
    }

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can delete
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(ApiError::err_plain("no_comment_edit_allowed").into());
    }

    // Do the delete
    let deleted = data.deleted;
    let updated_comment = blocking(context.pool(), move |conn| {
      Comment::update_deleted(conn, comment_id, deleted)
    })
    .await?
    .map_err(|e| ApiError::err("couldnt_update_comment", e))?;

    let post_id = updated_comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let recipient_ids = send_local_notifs(
      vec![],
      &updated_comment,
      &local_user_view.person,
      &post,
      false,
      context,
    )
    .await?;

    // Send the apub message
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, orig_comment.post.community_id)
    })
    .await??;
    send_apub_delete(
      &local_user_view.person.clone().into(),
      &community.clone().into(),
      DeletableObjects::Comment(Box::new(updated_comment.into())),
      deleted,
      context,
    )
    .await?;

    send_comment_ws_message(
      data.comment_id,
      UserOperationCrud::DeleteComment,
      websocket_id,
      None, // TODO a comment delete might clear forms?
      Some(local_user_view.person.id),
      recipient_ids,
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveComment {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &RemoveComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only a mod or admin can remove
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_comment.community.id,
    )
    .await?;

    // Do the remove
    let removed = data.removed;
    let updated_comment = blocking(context.pool(), move |conn| {
      Comment::update_removed(conn, comment_id, removed)
    })
    .await?
    .map_err(|e| ApiError::err("couldnt_update_comment", e))?;

    // Mod tables
    let form = ModRemoveCommentForm {
      mod_person_id: local_user_view.person.id,
      comment_id: data.comment_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
    };
    blocking(context.pool(), move |conn| {
      ModRemoveComment::create(conn, &form)
    })
    .await??;

    let post_id = updated_comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let recipient_ids = send_local_notifs(
      vec![],
      &updated_comment,
      &local_user_view.person.clone(),
      &post,
      false,
      context,
    )
    .await?;

    // Send the apub message
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, orig_comment.post.community_id)
    })
    .await??;
    send_apub_remove(
      &local_user_view.person.clone().into(),
      &community.into(),
      DeletableObjects::Comment(Box::new(updated_comment.into())),
      data.reason.clone().unwrap_or_else(|| "".to_string()),
      removed,
      context,
    )
    .await?;

    send_comment_ws_message(
      data.comment_id,
      UserOperationCrud::RemoveComment,
      websocket_id,
      None, // TODO maybe this might clear other forms
      Some(local_user_view.person.id),
      recipient_ids,
      context,
    )
    .await
  }
}
