use crate::{
  activity_lists::AnnouncableActivities,
  community::send_activity_in_community,
  protocol::voting::{
    undo_vote::UndoVote,
    vote::{Vote, VoteType},
  },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_utils::{
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
};
use lemmy_apub_objects::objects::{
  comment::ApubComment,
  community::ApubCommunity,
  person::ApubPerson,
  post::ApubPost,
  PostOrComment,
};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    activity::ActivitySendTargets,
    comment::{CommentActions, CommentLikeForm},
    community::Community,
    person::Person,
    post::{PostActions, PostLikeForm},
  },
  traits::Likeable,
};
use lemmy_utils::error::LemmyResult;

pub mod undo_vote;
pub mod vote;

pub(crate) async fn send_like_activity(
  object_id: DbUrl,
  actor: Person,
  community: Community,
  previous_score: Option<i16>,
  new_score: i16,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let object_id: ObjectId<PostOrComment> = object_id.into();
  let actor: ApubPerson = actor.into();
  let community: ApubCommunity = community.into();

  let empty = ActivitySendTargets::empty();
  // score of 1 means upvote, -1 downvote, 0 undo a previous vote
  if new_score != 0 {
    let vote = Vote::new(object_id, &actor, new_score.try_into()?, &context)?;
    let activity = AnnouncableActivities::Vote(vote);
    send_activity_in_community(activity, &actor, &community, empty, false, &context).await
  } else {
    let previous_vote_type = if previous_score.unwrap_or_default() > 0 {
      VoteType::Like
    } else {
      VoteType::Dislike
    };
    let vote = Vote::new(object_id, &actor, previous_vote_type, &context)?;
    let undo_vote = UndoVote::new(vote, &actor, &context)?;
    let activity = AnnouncableActivities::UndoVote(undo_vote);
    send_activity_in_community(activity, &actor, &community, empty, false, &context).await
  }
}

async fn vote_comment(
  vote_type: &VoteType,
  actor: ApubPerson,
  comment: &ApubComment,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let comment_id = comment.id;
  let mut like_form = CommentLikeForm::new(actor.id, comment_id, vote_type.into());
  let person_id = actor.id;
  comment.set_not_pending(&mut context.pool()).await?;
  CommentActions::remove_like(&mut context.pool(), person_id, comment_id).await?;
  like_form = plugin_hook_before("before_comment_vote", like_form).await?;
  let like = CommentActions::like(&mut context.pool(), &like_form).await?;
  plugin_hook_after("after_comment_vote", &like)?;
  Ok(())
}

async fn vote_post(
  vote_type: &VoteType,
  actor: ApubPerson,
  post: &ApubPost,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let post_id = post.id;
  let mut like_form = PostLikeForm::new(post.id, actor.id, vote_type.into());
  let person_id = actor.id;
  post.set_not_pending(&mut context.pool()).await?;
  PostActions::remove_like(&mut context.pool(), person_id, post_id).await?;
  like_form = plugin_hook_before("before_post_vote", like_form).await?;
  let like = PostActions::like(&mut context.pool(), &like_form).await?;
  plugin_hook_after("after_post_vote", &like)?;
  Ok(())
}

async fn undo_vote_comment(
  actor: ApubPerson,
  comment: &ApubComment,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let comment_id = comment.id;
  let person_id = actor.id;
  CommentActions::remove_like(&mut context.pool(), person_id, comment_id).await?;
  Ok(())
}

async fn undo_vote_post(
  actor: ApubPerson,
  post: &ApubPost,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let post_id = post.id;
  let person_id = actor.id;
  PostActions::remove_like(&mut context.pool(), person_id, post_id).await?;
  Ok(())
}
