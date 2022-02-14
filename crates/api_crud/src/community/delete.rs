use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, community::*, get_local_user_view_from_jwt, is_admin};
use lemmy_apub::activities::deletion::{send_apub_delete_in_community, DeletableObjects};
use lemmy_db_schema::{
  source::{
    community::Community,
    moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
use lemmy_utils::{utils::naive_from_unix, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};

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
    let community_mods = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    // Make sure deleter is the top mod
    if local_user_view.person.id != community_mods[0].moderator.id {
      return Err(LemmyError::from_message("no_community_edit_allowed"));
    }

    // Do the delete
    let community_id = data.community_id;
    let deleted = data.deleted;
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update_deleted(conn, community_id, deleted)
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_update_community"))?;

    let res = send_community_ws_message(
      data.community_id,
      UserOperationCrud::DeleteCommunity,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await?;

    // Send apub messages
    let deletable = DeletableObjects::Community(Box::new(updated_community.clone().into()));
    send_apub_delete_in_community(
      local_user_view.person,
      updated_community,
      deletable,
      None,
      deleted,
      context,
    )
    .await?;

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
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
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update_removed(conn, community_id, removed)
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_update_community"))?;

    // Mod tables
    let expires = data.expires.map(naive_from_unix);
    let form = ModRemoveCommunityForm {
      mod_person_id: local_user_view.person.id,
      community_id: data.community_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
      expires,
    };
    blocking(context.pool(), move |conn| {
      ModRemoveCommunity::create(conn, &form)
    })
    .await??;

    let res = send_community_ws_message(
      data.community_id,
      UserOperationCrud::RemoveCommunity,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await?;

    // Apub messages
    let deletable = DeletableObjects::Community(Box::new(updated_community.clone().into()));
    send_apub_delete_in_community(
      local_user_view.person,
      updated_community,
      deletable,
      data.reason.clone().or_else(|| Some("".to_string())),
      removed,
      context,
    )
    .await?;
    Ok(res)
  }
}
