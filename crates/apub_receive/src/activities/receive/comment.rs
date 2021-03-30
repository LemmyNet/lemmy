use crate::activities::receive::get_actor_as_person;
use activitystreams::{
  activity::{ActorAndObjectRefExt, Create, Dislike, Like, Update},
  base::ExtendsExt,
};
use anyhow::Context;
use lemmy_api_common::{blocking, comment::CommentResponse, send_local_notifs};
use lemmy_apub::{objects::FromApub, ActorType, NoteExt};
use lemmy_db_queries::{source::comment::Comment_, Crud, Likeable};
use lemmy_db_schema::source::{
  comment::{Comment, CommentLike, CommentLikeForm},
  post::Post,
};
use lemmy_db_views::comment_view::CommentView;
use lemmy_utils::{location_info, utils::scrape_text_for_mentions, LemmyError};
use lemmy_websocket::{messages::SendComment, LemmyContext, UserOperation, UserOperationCrud};

pub(crate) async fn receive_create_comment(
  create: Create,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = get_actor_as_person(&create, context, request_counter).await?;
  let note = NoteExt::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment =
    Comment::from_apub(&note, context, person.actor_id(), request_counter, false).await?;

  let post_id = comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&comment.content);
  let recipient_ids = send_local_notifs(
    mentions,
    comment.clone(),
    person,
    post,
    context.pool(),
    true,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment.id, None)
  })
  .await??;

  let res = CommentResponse {
    comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperationCrud::CreateComment,
    comment: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_update_comment(
  update: Update,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let note = NoteExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let person = get_actor_as_person(&update, context, request_counter).await?;

  let comment =
    Comment::from_apub(&note, context, person.actor_id(), request_counter, false).await?;

  let comment_id = comment.id;
  let post_id = comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&comment.content);
  let recipient_ids =
    send_local_notifs(mentions, comment, person, post, context.pool(), false).await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;

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

pub(crate) async fn receive_like_comment(
  like: Like,
  comment: Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = get_actor_as_person(&like, context, request_counter).await?;

  let comment_id = comment.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    person_id: person.id,
    score: 1,
  };
  let person_id = person.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, person_id, comment_id)?;
    CommentLike::like(conn, &like_form)
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

pub(crate) async fn receive_dislike_comment(
  dislike: Dislike,
  comment: Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = get_actor_as_person(&dislike, context, request_counter).await?;

  let comment_id = comment.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    person_id: person.id,
    score: -1,
  };
  let person_id = person.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, person_id, comment_id)?;
    CommentLike::like(conn, &like_form)
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

pub(crate) async fn receive_delete_comment(
  context: &LemmyContext,
  comment: Comment,
) -> Result<(), LemmyError> {
  let deleted_comment = blocking(context.pool(), move |conn| {
    Comment::update_deleted(conn, comment.id, true)
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

pub(crate) async fn receive_remove_comment(
  context: &LemmyContext,
  comment: Comment,
) -> Result<(), LemmyError> {
  let removed_comment = blocking(context.pool(), move |conn| {
    Comment::update_removed(conn, comment.id, true)
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
