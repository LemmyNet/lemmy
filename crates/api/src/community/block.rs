use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::community::{CommunityActions, CommunityBlockForm},
  traits::{Blockable, Followable},
  utils::get_conn,
};
use lemmy_db_views_community::{
  api::{BlockCommunity, BlockCommunityResponse},
  CommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn user_block_community(
  data: Json<BlockCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BlockCommunityResponse>> {
  let community_id = data.community_id;
  let person_id = local_user_view.person.id;
  let community_block_form = CommunityBlockForm::new(community_id, person_id);

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  conn
    .run_transaction(|conn| {
      async move {
        if tx_data.block {
          CommunityActions::block(&mut conn.into(), &community_block_form).await?;

          // Also, unfollow the community, and send a federated unfollow
          CommunityActions::unfollow(&mut conn.into(), person_id, tx_data.community_id)
            .await
            .ok();
        } else {
          CommunityActions::unblock(&mut conn.into(), &community_block_form).await?;
        }

        Ok(())
      }
      .scope_boxed()
    })
    .await?;

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
