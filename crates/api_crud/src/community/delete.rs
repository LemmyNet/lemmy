use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{community::*, get_local_user_view_from_jwt, is_admin};
use lemmy_apub::activities::deletion::{send_apub_delete, send_apub_remove};
use lemmy_db_schema::{
  source::{
    community::Community,
    moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
use lemmy_utils::{utils::naive_from_unix, ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteCommunity {
  type Response = CommunityResponse;

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
    let community_mods = context
      .conn()
      .await?
      .interact(move |conn| CommunityModeratorView::for_community(conn, community_id))
      .await??;

    // Make sure deleter is the top mod
    if local_user_view.person.id != community_mods[0].moderator.id {
      return Err(ApiError::err_plain("no_community_edit_allowed").into());
    }

    // Do the delete
    let community_id = data.community_id;
    let deleted = data.deleted;
    let updated_community = context
      .conn()
      .await?
      .interact(move |conn| Community::update_deleted(conn, community_id, deleted))
      .await?
      .map_err(|e| ApiError::err("couldnt_update_community", e))?;

    // Send apub messages
    send_apub_delete(
      &local_user_view.person.clone().into(),
      &updated_community.clone().into(),
      updated_community.actor_id.clone().into(),
      deleted,
      context,
    )
    .await?;

    send_community_ws_message(
      data.community_id,
      UserOperationCrud::DeleteCommunity,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &RemoveCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Verify its an admin (only an admin can remove a community)
    is_admin(&local_user_view)?;

    // Do the remove
    let community_id = data.community_id;
    let removed = data.removed;
    let updated_community = context
      .conn()
      .await?
      .interact(move |conn| Community::update_removed(conn, community_id, removed))
      .await?
      .map_err(|e| ApiError::err("couldnt_update_community", e))?;

    // Mod tables
    let expires = data.expires.map(naive_from_unix);
    let form = ModRemoveCommunityForm {
      mod_person_id: local_user_view.person.id,
      community_id: data.community_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
      expires,
    };
    context
      .conn()
      .await?
      .interact(move |conn| ModRemoveCommunity::create(conn, &form))
      .await??;

    // Apub messages
    send_apub_remove(
      &local_user_view.person.clone().into(),
      &updated_community.clone().into(),
      updated_community.actor_id.clone().into(),
      data.reason.clone().unwrap_or_else(|| "".to_string()),
      removed,
      context,
    )
    .await?;

    send_community_ws_message(
      data.community_id,
      UserOperationCrud::RemoveCommunity,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
