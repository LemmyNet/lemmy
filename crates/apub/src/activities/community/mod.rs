use lemmy_api_common::{blocking, community::CommunityResponse};
use lemmy_db_schema::CommunityId;
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext};

pub mod add_mod;
pub mod announce;
pub mod block_user;
pub mod undo_block_user;
pub mod update;

pub(crate) async fn send_websocket_message<
  OP: ToString + Send + lemmy_websocket::OperationType + 'static,
>(
  community_id: CommunityId,
  op: OP,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community_view = blocking(context.pool(), move |conn| {
    CommunityView::read(conn, community_id, None)
  })
  .await??;

  let res = CommunityResponse { community_view };

  context.chat_server().do_send(SendCommunityRoomMessage {
    op,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}
