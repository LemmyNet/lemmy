use crate::{
  activities::receive::{announce_if_community_is_local, get_actor_as_user},
  fetcher::get_or_fetch_and_insert_comment,
  ActorType,
  FromApub,
};
use activitystreams::{
  activity::{ActorAndObjectRefExt, Create, Delete, Dislike, Like, Remove, Update},
  base::ExtendsExt,
  object::Note,
};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_db::{
  comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
  comment_view::CommentView,
  post::Post,
  Crud,
  Likeable,
};
use lemmy_structs::{blocking, comment::CommentResponse, send_local_notifs};
use lemmy_utils::{location_info, utils::scrape_text_for_mentions, LemmyError};
use lemmy_websocket::{messages::SendComment, LemmyContext, UserOperation};

pub(crate) async fn receive_create_comment(
  create: Create,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&create, context, request_counter).await?;
  let note = Note::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment =
    CommentForm::from_apub(&note, context, Some(user.actor_id()?), request_counter).await?;

  let inserted_comment =
    blocking(context.pool(), move |conn| Comment::upsert(conn, &comment)).await??;

  let post_id = inserted_comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&inserted_comment.content);
  let recipient_ids = send_local_notifs(
    mentions,
    inserted_comment.clone(),
    &user,
    post,
    context.pool(),
    true,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, inserted_comment.id, None)
  })
  .await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateComment,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(create, context, request_counter).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_update_comment(
  update: Update,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_actor_as_user(&update, context, request_counter).await?;

  let comment =
    CommentForm::from_apub(&note, context, Some(user.actor_id()?), request_counter).await?;

  let original_comment_id =
    get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context, request_counter)
      .await?
      .id;

  let updated_comment = blocking(context.pool(), move |conn| {
    Comment::update(conn, original_comment_id, &comment)
  })
  .await??;

  let post_id = updated_comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids = send_local_notifs(
    mentions,
    updated_comment,
    &user,
    post,
    context.pool(),
    false,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, original_comment_id, None)
  })
  .await??;

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

  announce_if_community_is_local(update, context, request_counter).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_like_comment(
  like: Like,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_actor_as_user(&like, context, request_counter).await?;

  let comment = CommentForm::from_apub(&note, context, None, request_counter).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context, request_counter)
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: 1,
  };
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)?;
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
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateCommentLike,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(like, context, request_counter).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_dislike_comment(
  dislike: Dislike,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;
  let user = get_actor_as_user(&dislike, context, request_counter).await?;

  let comment = CommentForm::from_apub(&note, context, None, request_counter).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context, request_counter)
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: -1,
  };
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)?;
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
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateCommentLike,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(dislike, context, request_counter).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_delete_comment(
  context: &LemmyContext,
  delete: Delete,
  comment: Comment,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
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
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };
  context.chat_server().do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(delete, context, request_counter).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_remove_comment(
  context: &LemmyContext,
  _remove: Remove,
  comment: Comment,
) -> Result<HttpResponse, LemmyError> {
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
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };
  context.chat_server().do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}
