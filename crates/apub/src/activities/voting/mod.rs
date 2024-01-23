use crate::{
  activities::community::send_activity_in_community,
  activity_lists::AnnouncableActivities,
  fetcher::post_or_comment::PostOrComment,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::activities::voting::{
    undo_vote::UndoVote,
    vote::{Vote, VoteType},
  },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl},
  source::{
    activity::ActivitySendTargets,
    comment::{CommentLike, CommentLikeForm},
    community::Community,
    local_site::LocalSite,
    person::Person,
    post::{Post, PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::error::{LemmyError, LemmyResult};

pub mod undo_vote;
pub mod vote;

pub(crate) async fn send_like_activity(
  object_id: DbUrl,
  actor: Person,
  community: Community,
  score: i16,
  context: Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let object_id: ObjectId<PostOrComment> = object_id.into();
  let actor: ApubPerson = actor.into();
  let community: ApubCommunity = community.into();

  let empty = ActivitySendTargets::empty();
  // score of 1 means upvote, -1 downvote, 0 undo a previous vote
  if score != 0 {
    let vote = Vote::new(object_id, &actor, &community, score.try_into()?, &context)?;
    let activity = AnnouncableActivities::Vote(vote);
    send_activity_in_community(activity, &actor, &community, empty, false, &context).await
  } else {
    // Lemmy API doesnt distinguish between Undo/Like and Undo/Dislike, so we hardcode it here.
    let vote = Vote::new(object_id, &actor, &community, VoteType::Like, &context)?;
    let undo_vote = UndoVote::new(vote, &actor, &community, &context)?;
    let activity = AnnouncableActivities::UndoVote(undo_vote);
    send_activity_in_community(activity, &actor, &community, empty, false, &context).await
  }
}

#[tracing::instrument(skip_all)]
async fn vote_comment(
  vote_type: &VoteType,
  actor: ApubPerson,
  comment: &ApubComment,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let comment_id = comment.id;
  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    person_id: actor.id,
    score: vote_type.into(),
  };
  let post = Post::read(&mut context.pool(), comment.post_id).await?;
  check_vote_permission(Some(vote_type), &actor, post.community_id, context).await?;
  CommentLike::remove(&mut context.pool(), actor.id, comment_id).await?;
  CommentLike::like(&mut context.pool(), &like_form).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn vote_post(
  vote_type: &VoteType,
  actor: ApubPerson,
  post: &ApubPost,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let post_id = post.id;
  let like_form = PostLikeForm {
    post_id: post.id,
    person_id: actor.id,
    score: vote_type.into(),
  };

  check_vote_permission(Some(vote_type), &actor, post.community_id, context).await?;
  PostLike::remove(&mut context.pool(), actor.id, post_id).await?;
  PostLike::like(&mut context.pool(), &like_form).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn undo_vote_comment(
  actor: ApubPerson,
  comment: &ApubComment,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let post = Post::read(&mut context.pool(), comment.post_id).await?;
  check_vote_permission(None, &actor, post.community_id, context).await?;
  CommentLike::remove(&mut context.pool(), actor.id, comment.id).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn undo_vote_post(
  actor: ApubPerson,
  post: &ApubPost,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  check_vote_permission(None, &actor, post.community_id, context).await?;
  PostLike::remove(&mut context.pool(), actor.id, post.id).await?;
  Ok(())
}

pub async fn check_vote_permission(
  vote_type: Option<&VoteType>,
  person: &Person,
  community_id: CommunityId,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  let community = Community::read(&mut context.pool(), community_id).await?;
  let score = vote_type.map(std::convert::Into::into).unwrap_or(0);
  lemmy_api_common::utils::check_vote_permission(score, &local_site, person, &community, context)
    .await?;
  Ok(())
}
