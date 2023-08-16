use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::{BlockCommunity, BlockCommunityResponse},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    community_block::{CommunityBlock, CommunityBlockForm},
  },
  traits::{Blockable, Followable},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn block_community(
  data: Json<BlockCommunity>,
  context: Data<LemmyContext>,
) -> Result<Json<BlockCommunityResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let community_id = data.community_id;
  let person_id = local_user_view.person.id;
  let community_block_form = CommunityBlockForm {
    person_id,
    community_id,
  };

  if data.block {
    CommunityBlock::block(&mut context.pool(), &community_block_form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityBlockAlreadyExists)?;

    // Also, unfollow the community, and send a federated unfollow
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id,
      pending: false,
    };

    CommunityFollower::unfollow(&mut context.pool(), &community_follower_form)
      .await
      .ok();
  } else {
    CommunityBlock::unblock(&mut context.pool(), &community_block_form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityBlockAlreadyExists)?;
  }

  let community_view =
    CommunityView::read(&mut context.pool(), community_id, Some(person_id), false).await?;

  ActivityChannel::submit_activity(
    SendActivityData::FollowCommunity(
      community_view.community.clone(),
      local_user_view.person.clone(),
      false,
    ),
    &context,
  )
  .await?;

  Ok(Json(BlockCommunityResponse {
    blocked: data.block,
    community_view,
  }))
}
