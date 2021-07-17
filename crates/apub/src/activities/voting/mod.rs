use crate::{
  activities::{
    comment::send_websocket_message as send_comment_message,
    post::send_websocket_message as send_post_message,
  },
  fetcher::{
    objects::get_or_fetch_and_insert_post_or_comment,
    person::get_or_fetch_and_upsert_person,
  },
  PostOrComment,
};
use lemmy_api_common::blocking;
use lemmy_db_queries::Likeable;
use lemmy_db_schema::source::{
  comment::{Comment, CommentLike, CommentLikeForm},
  post::{Post, PostLike, PostLikeForm},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperation};
use std::ops::Deref;
use url::Url;

pub mod dislike;
pub mod like;
pub mod undo_dislike;
pub mod undo_like;

pub(in crate::activities::voting) async fn receive_like_or_dislike(
  score: i16,
  actor: &Url,
  object: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  match get_or_fetch_and_insert_post_or_comment(object, context, request_counter).await? {
    PostOrComment::Post(p) => {
      like_or_dislike_post(score, actor, p.deref(), context, request_counter).await
    }
    PostOrComment::Comment(c) => {
      like_or_dislike_comment(score, actor, c.deref(), context, request_counter).await
    }
  }
}

async fn like_or_dislike_comment(
  score: i16,
  actor: &Url,
  comment: &Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let actor = get_or_fetch_and_upsert_person(actor, context, request_counter).await?;

  let comment_id = comment.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    person_id: actor.id,
    score,
  };
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, person_id, comment_id)?;
    CommentLike::like(conn, &like_form)
  })
  .await??;

  send_comment_message(
    comment_id,
    vec![],
    UserOperation::CreateCommentLike,
    context,
  )
  .await
}

async fn like_or_dislike_post(
  score: i16,
  actor: &Url,
  post: &Post,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let actor = get_or_fetch_and_upsert_person(actor, context, request_counter).await?;

  let post_id = post.id;
  let like_form = PostLikeForm {
    post_id: post.id,
    person_id: actor.id,
    score,
  };
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, person_id, post_id)?;
    PostLike::like(conn, &like_form)
  })
  .await??;

  send_post_message(post.id, UserOperation::CreatePostLike, context).await
}

pub(in crate::activities::voting) async fn receive_undo_like_or_dislike(
  actor: &Url,
  object: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  match get_or_fetch_and_insert_post_or_comment(object, context, request_counter).await? {
    PostOrComment::Post(p) => {
      undo_like_or_dislike_post(actor, p.deref(), context, request_counter).await
    }
    PostOrComment::Comment(c) => {
      undo_like_or_dislike_comment(actor, c.deref(), context, request_counter).await
    }
  }
}

async fn undo_like_or_dislike_comment(
  actor: &Url,
  comment: &Comment,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let actor = get_or_fetch_and_upsert_person(actor, context, request_counter).await?;

  let comment_id = comment.id;
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, person_id, comment_id)
  })
  .await??;

  send_comment_message(
    comment.id,
    vec![],
    UserOperation::CreateCommentLike,
    context,
  )
  .await
}

async fn undo_like_or_dislike_post(
  actor: &Url,
  post: &Post,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let actor = get_or_fetch_and_upsert_person(actor, context, request_counter).await?;

  let post_id = post.id;
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, person_id, post_id)
  })
  .await??;
  send_post_message(post.id, UserOperation::CreatePostLike, context).await
}
