use crate::UserOperation;
use actix::{prelude::*, Recipient};
use lemmy_api_common::{comment::CommentResponse, post::PostResponse};
use lemmy_db_schema::{CommunityId, LocalUserId, PostId};
use lemmy_utils::{ConnectionId, IpAddr};
use serde::{Deserialize, Serialize};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
  pub addr: Recipient<WsMessage>,
  pub ip: IpAddr,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub id: ConnectionId,
  pub ip: IpAddr,
}

/// The messages sent to websocket clients
#[derive(Serialize, Deserialize, Message)]
#[rtype(result = "Result<String, std::convert::Infallible>")]
pub struct StandardMessage {
  /// Id of the client session
  pub id: ConnectionId,
  /// Peer message
  pub msg: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendAllMessage<OP: ToString, Response> {
  pub op: OP,
  pub response: Response,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUserRoomMessage<OP: ToString, Response> {
  pub op: OP,
  pub response: Response,
  pub local_recipient_id: LocalUserId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommunityRoomMessage<OP: ToString, Response> {
  pub op: OP,
  pub response: Response,
  pub community_id: CommunityId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendModRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub community_id: CommunityId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct SendPost<OP: ToString> {
  pub op: OP,
  pub post: PostResponse,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct SendComment<OP: ToString> {
  pub op: OP,
  pub comment: CommentResponse,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinUserRoom {
  pub local_user_id: LocalUserId,
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinCommunityRoom {
  pub community_id: CommunityId,
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinModRoom {
  pub community_id: CommunityId,
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinPostRoom {
  pub post_id: PostId,
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(usize)]
pub struct GetUsersOnline;

#[derive(Message)]
#[rtype(usize)]
pub struct GetPostUsersOnline {
  pub post_id: PostId,
}

#[derive(Message)]
#[rtype(usize)]
pub struct GetCommunityUsersOnline {
  pub community_id: CommunityId,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CaptchaItem {
  pub uuid: String,
  pub answer: String,
  pub expires: chrono::NaiveDateTime,
}

#[derive(Message)]
#[rtype(bool)]
pub struct CheckCaptcha {
  pub uuid: String,
  pub answer: String,
}
