use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, DeleteComment},
  utils::{blocking, check_community_ban, get_local_user_view_from_jwt},
};
use lemmy_apub::activities::deletion::{send_apub_delete_in_community, DeletableObjects};
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, post::Post},
  traits::Crud,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{
  send::{send_comment_ws_message, send_local_notifs},
  LemmyContext,
  UserOperationCrud,
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, websocket_id))]
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
      return Err(LemmyError::from_message("couldnt_update_comment"));
    }

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can delete
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(LemmyError::from_message("no_comment_edit_allowed"));
    }

    // Do the delete
    let deleted = data.deleted;
    let updated_comment = blocking(context.pool(), move |conn| {
      Comment::update_deleted(conn, comment_id, deleted)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

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

    let res = send_comment_ws_message(
      data.comment_id,
      UserOperationCrud::DeleteComment,
      websocket_id,
      None, // TODO a comment delete might clear forms?
      Some(local_user_view.person.id),
      recipient_ids,
      context,
    )
    .await?;

    // Send the apub message
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, orig_comment.post.community_id)
    })
    .await??;
    let deletable = DeletableObjects::Comment(Box::new(updated_comment.clone().into()));
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
