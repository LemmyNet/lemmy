use crate::{
  activities::receive::get_actor_as_user,
  fetcher::get_or_fetch_and_insert_post,
  FromApub,
  PageExt,
};
use activitystreams::{activity::*, prelude::*};
use anyhow::Context;
use lemmy_db::{
  post::{Post, PostForm, PostLike},
  post_view::PostView,
  Likeable,
};
use lemmy_structs::{blocking, post::PostResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendPost, LemmyContext, UserOperation};

pub(crate) async fn receive_undo_like_post(
  like: &Like,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(like, context, request_counter).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None, request_counter).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context, request_counter)
    .await?
    .id;

  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_dislike_post(
  dislike: &Dislike,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(dislike, context, request_counter).await?;
  let page = PageExt::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None, request_counter).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context, request_counter)
    .await?
    .id;

  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_delete_post(
  context: &LemmyContext,
  post: Post,
) -> Result<(), LemmyError> {
  let deleted_post = blocking(context.pool(), move |conn| {
    Post::update_deleted(conn, post.id, false)
  })
  .await??;

  // Refetch the view
  let post_id = deleted_post.id;
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };
  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_remove_post(
  context: &LemmyContext,
  post: Post,
) -> Result<(), LemmyError> {
  let removed_post = blocking(context.pool(), move |conn| {
    Post::update_removed(conn, post.id, false)
  })
  .await??;

  // Refetch the view
  let post_id = removed_post.id;
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  Ok(())
}
