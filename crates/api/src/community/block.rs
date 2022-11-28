use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{BlockCommunity, BlockCommunityResponse},
  context::LemmyContext,
  utils::get_local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    community_block::{CommunityBlock, CommunityBlockForm},
  },
  traits::{Blockable, Followable},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for BlockCommunity {
  type Response = BlockCommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<BlockCommunityResponse, LemmyError> {
    let data: &BlockCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_block_form = CommunityBlockForm {
      person_id,
      community_id,
    };

    if data.block {
      CommunityBlock::block(context.pool(), &community_block_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_block_already_exists"))?;

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };

      CommunityFollower::unfollow(context.pool(), &community_follower_form)
        .await
        .ok();
    } else {
      CommunityBlock::unblock(context.pool(), &community_block_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_block_already_exists"))?;
    }

    let community_view = CommunityView::read(context.pool(), community_id, Some(person_id)).await?;

    Ok(BlockCommunityResponse {
      blocked: data.block,
      community_view,
    })
  }
}
