use crate::activities::receive::get_actor_as_user;
use activitystreams::activity::{Dislike, Like};
use lemmy_db_queries::{source::comment::Comment_, Likeable};
use lemmy_db_schema::source::comment::{Comment, CommentLike};
use lemmy_db_views::comment_view::CommentView;
use lemmy_structs::{blocking, comment::CommentResponse};
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendComment, LemmyContext, UserOperation};

pub(crate) async fn receive_undo_like_comment(
  like: &Like,
  comment: Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(like, context, request_counter).await?;

  let comment_id = comment.id;
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)
  })
  .await??;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateCommentLike,
    comment: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_dislike_comment(
  dislike: &Dislike,
  comment: Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(dislike, context, request_counter).await?;

  let comment_id = comment.id;
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)
  })
  .await??;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateCommentLike,
    comment: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_delete_comment(
  context: &LemmyContext,
  comment: Comment,
) -> Result<(), LemmyError> {
  let deleted_comment = blocking(context.pool(), move |conn| {
    Comment::update_deleted(conn, comment.id, false)
  })
  .await??;

  // Refetch the view
  let comment_id = deleted_comment.id;
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_remove_comment(
  context: &LemmyContext,
  comment: Comment,
) -> Result<(), LemmyError> {
  let removed_comment = blocking(context.pool(), move |conn| {
    Comment::update_removed(conn, comment.id, false)
  })
  .await??;

  // Refetch the view
  let comment_id = removed_comment.id;
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    websocket_id: None,
  });

  Ok(())
}
