use actix_web::web::Data;
use lemmy_api_common::community::CommunityResponse;
use lemmy_utils::ConnectionId;
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperationCrud};

mod create;
mod delete;
mod read;
mod update;

pub(in crate::community) fn send_community_websocket(
  res: &CommunityResponse,
  context: &Data<LemmyContext>,
  websocket_id: Option<ConnectionId>,
  op: UserOperationCrud,
) {
  // Strip out the person id and subscribed when sending to others
  let mut res_sent = res.clone();
  res_sent.community_view.subscribed = false;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op,
    response: res_sent,
    community_id: res.community_view.community.id,
    websocket_id,
  });
}
