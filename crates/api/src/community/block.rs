use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::{BlockCommunity, BlockCommunityResponse},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::community::{CommunityActions, CommunityBlockForm},
  traits::{Blockable, Followable},
};
use lemmy_db_views::structs::{CommunityView, LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn user_block_community(
  data: Json<BlockCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BlockCommunityResponse>> {
  let community_id = data.community_id;
  let person_id = local_user_view.person.id;
  let community_block_form = CommunityBlockForm::new(community_id, person_id);

  if data.block {
    CommunityActions::block(&mut context.pool(), &community_block_form).await?;

    // Also, unfollow the community, and send a federated unfollow
    CommunityActions::unfollow(&mut context.pool(), person_id, data.community_id)
      .await
      .ok();
  } else {
    CommunityActions::unblock(&mut context.pool(), &community_block_form).await?;
  }

  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(&local_user_view.local_user),
    false,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::FollowCommunity(
      community_view.community.clone(),
      local_user_view.person.clone(),
      false,
    ),
    &context,
  )?;

  Ok(Json(BlockCommunityResponse {
    blocked: data.block,
    community_view,
  }))
}
