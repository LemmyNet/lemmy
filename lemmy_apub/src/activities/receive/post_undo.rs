use crate::{
  activities::receive::{announce_if_community_is_local, get_actor_as_user},
  fetcher::get_or_fetch_and_insert_post,
  FromApub,
  PageExt,
};
use activitystreams::{activity::*, prelude::*};
use actix_web::HttpResponse;
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
  undo: Undo,
  like: &Like,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(like, context).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_dislike_post(
  undo: Undo,
  dislike: &Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(dislike, context).await?;
  let page = PageExt::from_any_base(
    dislike
      .object()
      .to_owned()
      .one()
      .context(location_info!())?,
  )?
  .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
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

  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_delete_post(
  context: &LemmyContext,
  undo: Undo,
  post: Post,
) -> Result<HttpResponse, LemmyError> {
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

  let user = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_remove_post(
  context: &LemmyContext,
  undo: Undo,
  post: Post,
) -> Result<HttpResponse, LemmyError> {
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

  let mod_ = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}
