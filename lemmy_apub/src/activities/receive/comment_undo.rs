use crate::{
  activities::receive::get_actor_as_user,
  fetcher::get_or_fetch_and_insert_comment,
  FromApub,
};
use activitystreams::{activity::*, object::Note, prelude::*};
use anyhow::Context;
use lemmy_db::{
  comment::{Comment, CommentForm, CommentLike},
  comment_view::CommentView,
  Likeable,
};
use lemmy_structs::{blocking, comment::CommentResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendComment, LemmyContext, UserOperation};

pub(crate) async fn receive_undo_like_comment(
  like: &Like,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(like, context, request_counter).await?;
  let note = Note::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, None, request_counter).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context, request_counter)
    .await?
    .id;

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
    comment: comment_view,
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
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(dislike, context, request_counter).await?;
  let note = Note::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, None, request_counter).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context, request_counter)
    .await?
    .id;

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
    comment: comment_view,
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
    comment: comment_view,
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
    comment: comment_view,
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
