use crate::{
  activities::community::send_activity_in_community,
  activity_lists::AnnouncableActivities,
  fetcher::post_or_comment::PostOrComment,
  objects::{comment::ApubComment, person::ApubPerson, post::ApubPost},
  protocol::activities::voting::{
    undo_vote::UndoVote,
    vote::{Vote, VoteType},
  },
  SendActivity,
};
use activitypub_federation::core::object_id::ObjectId;
use lemmy_api_common::{
  comment::{CommentResponse, CreateCommentLike},
  context::LemmyContext,
  post::{CreatePostLike, PostResponse},
  sensitive::Sensitive,
  utils::get_local_user_view_from_jwt,
  websocket::UserOperation,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{
    comment::{CommentLike, CommentLikeForm},
    community::Community,
    person::Person,
    post::{PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::error::LemmyError;

pub mod undo_vote;
pub mod vote;

#[async_trait::async_trait(?Send)]
impl SendActivity for CreatePostLike {
  type Response = PostResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object_id = ObjectId::new(response.post_view.post.ap_id.clone());
    let community_id = response.post_view.community.id;
    send_activity(
      object_id,
      community_id,
      request.score,
      &request.auth,
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl SendActivity for CreateCommentLike {
  type Response = CommentResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object_id = ObjectId::new(response.comment_view.comment.ap_id.clone());
    let community_id = response.comment_view.community.id;
    send_activity(
      object_id,
      community_id,
      request.score,
      &request.auth,
      context,
    )
    .await
  }
}

async fn send_activity(
  object_id: ObjectId<PostOrComment>,
  community_id: CommunityId,
  score: i16,
  jwt: &Sensitive<String>,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community = Community::read(context.pool(), community_id).await?.into();
  let local_user_view = get_local_user_view_from_jwt(jwt, context.pool(), context.secret()).await?;
  let actor = Person::read(context.pool(), local_user_view.person.id)
    .await?
    .into();

  // score of 1 means upvote, -1 downvote, 0 undo a previous vote
  if score != 0 {
    let vote = Vote::new(object_id, &actor, &community, score.try_into()?, context)?;
    let activity = AnnouncableActivities::Vote(vote);
    send_activity_in_community(activity, &actor, &community, vec![], false, context).await
  } else {
    // Lemmy API doesnt distinguish between Undo/Like and Undo/Dislike, so we hardcode it here.
    let vote = Vote::new(object_id, &actor, &community, VoteType::Like, context)?;
    let undo_vote = UndoVote::new(vote, &actor, &community, context)?;
    let activity = AnnouncableActivities::UndoVote(undo_vote);
    send_activity_in_community(activity, &actor, &community, vec![], false, context).await
  }
}

#[tracing::instrument(skip_all)]
async fn vote_comment(
  vote_type: &VoteType,
  actor: ApubPerson,
  comment: &ApubComment,
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
  CommentLike::remove(context.pool(), person_id, comment_id).await?;
  CommentLike::like(context.pool(), &like_form).await?;

  context
    .send_comment_ws_message_simple(&UserOperation::CreateCommentLike, comment_id)
    .await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn vote_post(
  vote_type: &VoteType,
  actor: ApubPerson,
  post: &ApubPost,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let post_id = post.id;
  let like_form = PostLikeForm {
    post_id: post.id,
    person_id: actor.id,
    score: vote_type.into(),
  };
  let person_id = actor.id;
  PostLike::remove(context.pool(), person_id, post_id).await?;
  PostLike::like(context.pool(), &like_form).await?;

  context
    .send_post_ws_message(&UserOperation::CreatePostLike, post.id, None, None)
    .await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn undo_vote_comment(
  actor: ApubPerson,
  comment: &ApubComment,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let comment_id = comment.id;
  let person_id = actor.id;
  CommentLike::remove(context.pool(), person_id, comment_id).await?;

  context
    .send_comment_ws_message_simple(&UserOperation::CreateCommentLike, comment_id)
    .await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn undo_vote_post(
  actor: ApubPerson,
  post: &ApubPost,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let post_id = post.id;
  let person_id = actor.id;
  PostLike::remove(context.pool(), person_id, post_id).await?;

  context
    .send_post_ws_message(&UserOperation::CreatePostLike, post_id, None, None)
    .await?;
  Ok(())
}
