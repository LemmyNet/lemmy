use crate::activities::receive::get_actor_as_user;
use activitystreams::activity::{Dislike, Like};
use lemmy_db_queries::{source::post::Post_, Likeable};
use lemmy_db_schema::source::post::{Post, PostLike};
use lemmy_db_views::post_view::PostView;
use lemmy_structs::{blocking, post::PostResponse};
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendPost, LemmyContext, UserOperation};

pub(crate) async fn receive_undo_like_post(
  like: &Like,
  post: Post,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(like, context, request_counter).await?;

  let post_id = post.id;
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

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_dislike_post(
  dislike: &Dislike,
  post: Post,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(dislike, context, request_counter).await?;

  let post_id = post.id;
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

  let res = PostResponse { post_view };

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

  let res = PostResponse { post_view };
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

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  Ok(())
}
