use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, DeleteCommunity},
  context::LemmyContext,
  utils::{get_local_user_view_from_jwt, is_top_mod},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::community::{Community, CommunityUpdateForm},
  traits::Crud,
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &DeleteCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Fetch the community mods
    let community_id = data.community_id;
    let community_mods =
      CommunityModeratorView::for_community(context.pool(), community_id).await?;

    // Make sure deleter is the top mod
    is_top_mod(&local_user_view, &community_mods)?;

    // Do the delete
    let community_id = data.community_id;
    let deleted = data.deleted;
    Community::update(
      context.pool(),
      community_id,
      &CommunityUpdateForm::builder()
        .deleted(Some(deleted))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community"))?;

    let res = context
      .send_community_ws_message(
        &UserOperationCrud::DeleteCommunity,
        data.community_id,
        websocket_id,
        Some(local_user_view.person.id),
      )
      .await?;

    Ok(res)
  }
}
