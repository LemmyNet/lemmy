use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{BlockCommunity, BlockCommunityResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_apub::protocol::activities::following::undo_follow::UndoFollowCommunity;
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityFollower, CommunityFollowerForm},
    community_block::{CommunityBlock, CommunityBlockForm},
  },
  traits::{Blockable, Crud, Followable},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

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
      let block = move |conn: &mut _| CommunityBlock::block(conn, &community_block_form);
      blocking(context.pool(), block)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "community_block_already_exists"))?;

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };
      blocking(context.pool(), move |conn: &mut _| {
        CommunityFollower::unfollow(conn, &community_follower_form)
      })
      .await?
      .ok();
      let community = blocking(context.pool(), move |conn| {
        Community::read(conn, community_id)
      })
      .await??;
      UndoFollowCommunity::send(&local_user_view.person.into(), &community.into(), context).await?;
    } else {
      let unblock = move |conn: &mut _| CommunityBlock::unblock(conn, &community_block_form);
      blocking(context.pool(), unblock)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "community_block_already_exists"))?;
    }

    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    Ok(BlockCommunityResponse {
      blocked: data.block,
      community_view,
    })
  }
}
