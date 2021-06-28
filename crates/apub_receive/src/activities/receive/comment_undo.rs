use activitystreams::activity::{Dislike, Like};
use lemmy_api_common::{blocking, comment::CommentResponse};
use lemmy_db_queries::{source::comment::Comment_, Likeable};
use lemmy_db_schema::source::comment::{Comment, CommentLike};
use lemmy_db_views::comment_view::CommentView;
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendComment, LemmyContext, UserOperation, UserOperationCrud};

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
    op: UserOperationCrud::EditComment,
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
    op: UserOperationCrud::EditComment,
    comment: res,
    websocket_id: None,
  });

  Ok(())
}
