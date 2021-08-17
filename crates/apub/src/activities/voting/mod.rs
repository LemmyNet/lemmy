use crate::activities::voting::vote::VoteType;
use lemmy_api_common::blocking;
use lemmy_db_queries::Likeable;
use lemmy_db_schema::source::{
  comment::{Comment, CommentLike, CommentLikeForm},
  person::Person,
  post::{Post, PostLike, PostLikeForm},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{
  send::{send_comment_ws_message_simple, send_post_ws_message},
  LemmyContext,
  UserOperation,
};

pub mod undo_vote;
pub mod vote;

async fn vote_comment(
  vote_type: &VoteType,
  actor: Person,
  comment: &Comment,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let comment_id = comment.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    person_id: actor.id,
    score: vote_type.into(),
  };
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, person_id, comment_id)?;
    CommentLike::like(conn, &like_form)
  })
  .await??;

  send_comment_ws_message_simple(comment_id, UserOperation::CreateCommentLike, context).await?;
  Ok(())
}

async fn vote_post(
  vote_type: &VoteType,
  actor: Person,
  post: &Post,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let post_id = post.id;
  let like_form = PostLikeForm {
    post_id: post.id,
    person_id: actor.id,
    score: vote_type.into(),
  };
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, person_id, post_id)?;
    PostLike::like(conn, &like_form)
  })
  .await??;

  send_post_ws_message(post.id, UserOperation::CreatePostLike, None, None, context).await?;
  Ok(())
}

async fn undo_vote_comment(
  actor: Person,
  comment: &Comment,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let comment_id = comment.id;
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, person_id, comment_id)
  })
  .await??;

  send_comment_ws_message_simple(comment_id, UserOperation::CreateCommentLike, context).await?;
  Ok(())
}

async fn undo_vote_post(
  actor: Person,
  post: &Post,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let post_id = post.id;
  let person_id = actor.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, person_id, post_id)
  })
  .await??;

  send_post_ws_message(post_id, UserOperation::CreatePostLike, None, None, context).await?;
  Ok(())
}
