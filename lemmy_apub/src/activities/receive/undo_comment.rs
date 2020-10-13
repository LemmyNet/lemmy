use crate::{
  activities::receive::{announce_if_community_is_local, get_actor_as_user},
  fetcher::get_or_fetch_and_insert_comment,
  FromApub,
};
use activitystreams::{activity::*, object::Note, prelude::*};
use actix_web::HttpResponse;
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
  undo: Undo,
  like: &Like,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(like, context).await?;
  let note = Note::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, None).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_dislike_comment(
  undo: Undo,
  dislike: &Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(dislike, context).await?;
  let note = Note::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, None).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_delete_comment(
  context: &LemmyContext,
  undo: Undo,
  comment: Comment,
) -> Result<HttpResponse, LemmyError> {
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

  let user = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_remove_comment(
  context: &LemmyContext,
  undo: Undo,
  comment: Comment,
) -> Result<HttpResponse, LemmyError> {
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

  let mod_ = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}
