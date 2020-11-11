use crate::UserOperation;
use actix::{prelude::*, Recipient};
use lemmy_structs::{comment::CommentResponse, post::PostResponse};
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
  /// The address
  pub addr: Recipient<WSMessage>,
  /// The IP
  pub ip: IPAddr,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  /// The connection id
  pub id: ConnectionId,
  /// The IP
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
/// Send a message to all
pub struct SendAllMessage<Response> {
  /// The user operation
  pub op: UserOperation,
  /// The response
  pub response: Response,
  /// The websocket id
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Send a message to a user
pub struct SendUserRoomMessage<Response> {
  /// The user operation
  pub op: UserOperation,
  /// The response
  pub response: Response,
  /// The recipient id
  pub recipient_id: UserId,
  /// The websocket id
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Send a message to a community
pub struct SendCommunityRoomMessage<Response> {
  /// The user operation
  pub op: UserOperation,
  /// The response
  pub response: Response,
  /// The community id
  pub community_id: CommunityId,
  /// The websocket id
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Send a message to a post room
pub struct SendPost {
  /// The user operation
  pub op: UserOperation,
  /// The post response
  pub post: PostResponse,
  /// The websocket id
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Send a comment
pub struct SendComment {
  /// The user operation
  pub op: UserOperation,
  /// The comment response
  pub comment: CommentResponse,
  /// The websocket id
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Join a user room
pub struct JoinUserRoom {
  /// The user id
  pub user_id: UserId,
  /// The websocket id
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Join a community room
pub struct JoinCommunityRoom {
  /// The community id
  pub community_id: CommunityId,
  /// The websocket id
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Join a post room
pub struct JoinPostRoom {
  /// The post id
  pub post_id: PostId,
  /// The websocket id
  pub id: ConnectionId,
}

#[derive(Message)]
#[rtype(usize)]
/// Get the number of online users
pub struct GetUsersOnline;

#[derive(Message)]
#[rtype(usize)]
/// Get the number of users in a post room
pub struct GetPostUsersOnline {
  /// The post id
  pub post_id: PostId,
}

#[derive(Message)]
#[rtype(usize)]
/// Get the number of users in a community room
pub struct GetCommunityUsersOnline {
  /// The community id
  pub community_id: CommunityId,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
/// A Captcha item
pub struct CaptchaItem {
  /// The UUID
  pub uuid: String,
  /// The captcha answer
  pub answer: String,
  /// The expires time
  pub expires: chrono::NaiveDateTime,
}

#[derive(Message)]
#[rtype(bool)]
/// Check the captcha
pub struct CheckCaptcha {
  /// The uuid
  pub uuid: String,
  /// The answer
  pub answer: String,
}
