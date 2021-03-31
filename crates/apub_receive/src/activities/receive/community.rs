use lemmy_api_common::{blocking, community::CommunityResponse};
use lemmy_db_queries::source::community::Community_;
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperationCrud};

pub(crate) async fn receive_delete_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, true)
  })
  .await??;

  let community_id = deleted_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_remove_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, true)
  })
  .await??;

  let community_id = removed_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_delete_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let deleted_community = blocking(context.pool(), move |conn| {
    Community::update_deleted(conn, community.id, false)
  })
  .await??;

  let community_id = deleted_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;
  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_remove_community(
  context: &LemmyContext,
  community: Community,
) -> Result<(), LemmyError> {
  let removed_community = blocking(context.pool(), move |conn| {
    Community::update_removed(conn, community.id, false)
  })
  .await??;

  let community_id = removed_community.id;
  let res = CommunityResponse {
    community_view: blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??,
  };

  let community_id = res.community_view.community.id;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op: UserOperationCrud::EditCommunity,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}
