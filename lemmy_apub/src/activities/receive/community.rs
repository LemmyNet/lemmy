use crate::activities::receive::{announce_if_community_is_local, get_actor_as_user};
use activitystreams::activity::{Delete, Remove, Undo};
use actix_web::HttpResponse;
use lemmy_db::{community::Community, community_view::CommunityView};
use lemmy_structs::{blocking, community::CommunityResponse};
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};

pub(crate) async fn receive_delete_community(
  context: &LemmyContext,
  delete: Delete,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, true)
  })
  .await??;

  let community_id = deleted_community.id;
  let res = CommunityResponse {
    community: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  let user = get_actor_as_user(&delete, context).await?;
  announce_if_community_is_local(delete, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_remove_community(
  context: &LemmyContext,
  _remove: Remove,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, true)
  })
  .await??;

  let community_id = removed_community.id;
  let res = CommunityResponse {
    community: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_delete_community(
  context: &LemmyContext,
  undo: Undo,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, false)
  })
  .await??;

  let community_id = deleted_community.id;
  let res = CommunityResponse {
    community: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  let user = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &user, context).await?;
  Ok(HttpResponse::Ok().finish())
}

pub(crate) async fn receive_undo_remove_community(
  context: &LemmyContext,
  undo: Undo,
  community: Community,
) -> Result<HttpResponse, LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, false)
  })
  .await??;

  let community_id = removed_community.id;
  let res = CommunityResponse {
    community: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community.id;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperation::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  let mod_ = get_actor_as_user(&undo, context).await?;
  announce_if_community_is_local(undo, &mod_, context).await?;
  Ok(HttpResponse::Ok().finish())
}
