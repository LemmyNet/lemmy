use crate::{
  activities::receive::{announce_if_community_is_local, get_actor_as_user},
  fetcher::get_or_fetch_and_insert_post,
  ActorType,
  FromApub,
  PageExt,
};
use activitystreams::{
  activity::{Create, Delete, Dislike, Like, Remove, Update},
  prelude::*,
};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_db::{
  post::{Post, PostForm, PostLike, PostLikeForm},
  post_view::PostView,
  Crud,
  Likeable,
};
use lemmy_structs::{blocking, post::PostResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendPost, LemmyContext, UserOperation};

pub(crate) async fn receive_create_post(
  create: Create,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&create, context).await?;
  let page = PageExt::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, Some(user.actor_id()?)).await?;

  // Using an upsert, since likes (which fetch the post), sometimes come in before the create
  // resulting in double posts.
  let inserted_post = blocking(context.pool(), move |conn| Post::upsert(conn, &post)).await??;

  // Refetch the view
  let inserted_post_id = inserted_post.id;
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, inserted_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(create, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_update_post(
  update: Update,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&update, context).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, Some(user.actor_id()?)).await?;

  let original_post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
    .await?
    .id;

  blocking(context.pool(), move |conn| {
    Post::update(conn, original_post_id, &post)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, original_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(update, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_like_post(
  like: Like,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&like, context).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
    .await?
    .id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: 1,
  };
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)?;
    PostLike::like(conn, &like_form)
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

  announce_if_community_is_local(like, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_dislike_post(
  dislike: Dislike,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_actor_as_user(&dislike, context).await?;
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

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: -1,
  };
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)?;
    PostLike::like(conn, &like_form)
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

  announce_if_community_is_local(dislike, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_delete_post(
  context: &LemmyContext,
  delete: Delete,
  post: Post,
) -> Result<HttpResponse, LemmyError> {
  let deleted_post = blocking(context.pool(), move |conn| {
    Post::update_deleted(conn, post.id, true)
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

  let user = get_actor_as_user(&delete, context).await?;
  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_remove_post(
  context: &LemmyContext,
  _remove: Remove,
  post: Post,
) -> Result<HttpResponse, LemmyError> {
  let removed_post = blocking(context.pool(), move |conn| {
    Post::update_removed(conn, post.id, true)
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

  Ok(HttpResponse::Ok().finish())
}
