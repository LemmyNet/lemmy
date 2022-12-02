use crate::{
  objects::community::ApubCommunity,
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
  SendActivity,
};
use lemmy_api_common::{
  community::{CommunityResponse, FollowCommunity},
  context::LemmyContext,
  utils::get_local_user_view_from_jwt,
};
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_utils::error::LemmyError;

pub mod accept;
pub mod follow;
pub mod undo_follow;

#[async_trait::async_trait(?Send)]
impl SendActivity for FollowCommunity {
  type Response = CommunityResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    let person = local_user_view.person.clone().into();
    let community: ApubCommunity = Community::read(context.pool(), request.community_id)
      .await?
      .into();
    if community.local {
      Ok(())
    } else if request.follow {
      Follow::send(&person, &community, context).await
    } else {
      UndoFollow::send(&person, &community, context).await
    }
  }
}
