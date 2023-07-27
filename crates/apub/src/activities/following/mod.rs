use crate::{
  objects::community::ApubCommunity,
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
};
use activitypub_federation::config::Data;
use lemmy_api_common::{
  community::FollowCommunity,
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_utils::error::LemmyError;

pub mod accept;
pub mod follow;
pub mod undo_follow;

pub async fn send_follow_community(
  follow_community: &FollowCommunity,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let local_user_view = local_user_view_from_jwt(follow_community.auth.as_ref(), context).await?;
  let person = local_user_view.person.clone().into();
  let community: ApubCommunity =
    Community::read(&mut context.pool(), follow_community.community_id)
      .await?
      .into();
  if follow_community.follow {
    Follow::send(&person, &community, context).await
  } else {
    UndoFollow::send(&person, &community, context).await
  }
}
