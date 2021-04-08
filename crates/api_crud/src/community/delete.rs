use crate::{community::send_community_websocket, PerformCrud};
use actix_web::web::Data;
use lemmy_api_common::{blocking, community::*, get_local_user_view_from_jwt, is_admin};
use lemmy_apub::CommunityType;
use lemmy_db_queries::{source::community::Community_, Crud};
use lemmy_db_schema::source::{
  community::*,
  moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_utils::{utils::naive_from_unix, ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &DeleteCommunity = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Fetch the community mods
    let community_id = data.community_id;
    let community_mods = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    // Make sure deleter is the top mod
    if local_user_view.person.id != community_mods[0].moderator.id {
      return Err(ApiError::err("no_community_edit_allowed").into());
    }

    // Do the delete
    let community_id = data.community_id;
    let deleted = data.deleted;
    let updated_community = match blocking(context.pool(), move |conn| {
      Community::update_deleted(conn, community_id, deleted)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(ApiError::err("couldnt_update_community").into()),
    };

    // Send apub messages
    if deleted {
      updated_community.send_delete(context).await?;
    } else {
      updated_community.send_undo_delete(context).await?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    let res = CommunityResponse { community_view };

    send_community_websocket(
      &res,
      context,
      websocket_id,
      UserOperationCrud::DeleteCommunity,
    );

    Ok(res)
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
    let data: &RemoveCommunity = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Verify its an admin (only an admin can remove a community)
    is_admin(&local_user_view)?;

    // Do the remove
    let community_id = data.community_id;
    let removed = data.removed;
    let updated_community = match blocking(context.pool(), move |conn| {
      Community::update_removed(conn, community_id, removed)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(ApiError::err("couldnt_update_community").into()),
    };

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

    // Apub messages
    if removed {
      updated_community.send_remove(context).await?;
    } else {
      updated_community.send_undo_remove(context).await?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    let res = CommunityResponse { community_view };

    send_community_websocket(
      &res,
      context,
      websocket_id,
      UserOperationCrud::RemoveCommunity,
    );

    Ok(res)
  }
}
