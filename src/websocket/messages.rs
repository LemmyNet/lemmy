use crate::websocket::UserOperation;
use actix::{prelude::*, Recipient};
use lemmy_api_structs::{comment::CommentResponse, post::PostResponse};
use lemmy_utils::{CommunityId, ConnectionId, IPAddr, PostId, UserId};
use serde::{Deserialize, Serialize};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct WSMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
  pub addr: Recipient<WSMessage>,
  pub ip: IPAddr,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub id: ConnectionId,
  pub ip: IPAddr,
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
pub struct SendAllMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUserRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub recipient_id: UserId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommunityRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub community_id: CommunityId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPost {
  pub op: UserOperation,
  pub post: PostResponse,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendComment {
  pub op: UserOperation,
  pub comment: CommentResponse,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinUserRoom {
  pub user_id: UserId,
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
