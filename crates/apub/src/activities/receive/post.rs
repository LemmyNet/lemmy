use crate::{
  activities::receive::get_actor_as_user,
  inbox::receive_for_community::verify_mod_activity,
  objects::FromApub,
  ActorType,
  PageExt,
};
use activitystreams::{
  activity::{Announce, Create, Dislike, Like, Remove, Update},
  prelude::*,
};
use anyhow::Context;
use lemmy_api_structs::{blocking, post::PostResponse};
use lemmy_db_queries::{source::post::Post_, ApubObject, Crud, Likeable};
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{Post, PostLike, PostLikeForm},
  },
  DbUrl,
};
use lemmy_db_views::post_view::PostView;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendPost, LemmyContext, UserOperation};

pub(crate) async fn receive_create_post(
  create: Create,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(&create, context, request_counter).await?;
  let page = PageExt::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = Post::from_apub(&page, context, user.actor_id(), request_counter, false).await?;

  // Refetch the view
  let post_id = post.id;
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_update_post(
  update: Update,
  announce: Option<Announce>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(&update, context, request_counter).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post_id: DbUrl = page
    .id_unchecked()
    .context(location_info!())?
    .to_owned()
    .into();
  let old_post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, &post_id)
  })
  .await??;

  // If sticked or locked state was changed, make sure the actor is a mod
  let stickied = page.ext_one.stickied.context(location_info!())?;
  let locked = !page.ext_one.comments_enabled.context(location_info!())?;
  let mut is_mod_action = false;
  if stickied != old_post.stickied || locked != old_post.locked {
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, old_post.community_id)
    })
    .await??;
    verify_mod_activity(&update, announce, &community, context).await?;
    is_mod_action = true;
  }

  let post = Post::from_apub(
    &page,
    context,
    user.actor_id(),
    request_counter,
    is_mod_action,
  )
  .await?;

  let post_id = post.id;
  // Refetch the view
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

pub(crate) async fn receive_like_post(
  like: Like,
  post: Post,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(&like, context, request_counter).await?;

  let post_id = post.id;
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

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_dislike_post(
  dislike: Dislike,
  post: Post,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let user = get_actor_as_user(&dislike, context, request_counter).await?;

  let post_id = post.id;
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

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_delete_post(
  context: &LemmyContext,
  post: Post,
) -> Result<(), LemmyError> {
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

  let res = PostResponse { post_view };
  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_remove_post(
  context: &LemmyContext,
  _remove: Remove,
  post: Post,
) -> Result<(), LemmyError> {
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

  let res = PostResponse { post_view };
  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  Ok(())
}
