use crate::websocket::chat_server::ChatServer;
use actix::{Context, Handler, Message};
use lemmy_db_schema::newtypes::{CommunityId, LocalUserId, PostId};
use lemmy_utils::ConnectionId;
use std::collections::HashSet;

/// Sending a post room message
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPostRoomMessage {
  pub post_id: PostId,
  pub message: String,
  pub websocket_id: Option<ConnectionId>,
}

impl Handler<SendPostRoomMessage> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendPostRoomMessage, _: &mut Context<Self>) -> Self::Result {
    let room_connections = self.post_rooms.get(&msg.post_id);
    if let Some(connections) = room_connections {
      self.send_message(connections, &msg.message, msg.websocket_id);
    }
  }
}

/// Sending a community room message
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommunityRoomMessage {
  pub community_id: CommunityId,
  pub message: String,
  pub websocket_id: Option<ConnectionId>,
}

impl Handler<SendCommunityRoomMessage> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendCommunityRoomMessage, _: &mut Context<Self>) -> Self::Result {
    let room_connections = self.community_rooms.get(&msg.community_id);
    if let Some(connections) = room_connections {
      self.send_message(connections, &msg.message, msg.websocket_id);
    }
  }
}

/// Sending a mod room message
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendModRoomMessage {
  pub community_id: CommunityId,
  pub message: String,
  pub websocket_id: Option<ConnectionId>,
}

impl Handler<SendModRoomMessage> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendModRoomMessage, _: &mut Context<Self>) -> Self::Result {
    let room_connections = self.community_rooms.get(&msg.community_id);
    if let Some(connections) = room_connections {
      self.send_message(connections, &msg.message, msg.websocket_id);
    }
  }
}

/// Sending a user room message
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUserRoomMessage {
  pub recipient_id: LocalUserId,
  pub message: String,
  pub websocket_id: Option<ConnectionId>,
}

impl Handler<SendUserRoomMessage> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendUserRoomMessage, _: &mut Context<Self>) -> Self::Result {
    let room_connections = self.user_rooms.get(&msg.recipient_id);
    if let Some(connections) = room_connections {
      self.send_message(connections, &msg.message, msg.websocket_id);
    }
  }
}

/// Sending a message to every session
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendAllMessage {
  pub message: String,
  pub websocket_id: Option<ConnectionId>,
}

impl Handler<SendAllMessage> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendAllMessage, _: &mut Context<Self>) -> Self::Result {
    let connections: HashSet<ConnectionId> = self.sessions.keys().cloned().collect();
    self.send_message(&connections, &msg.message, msg.websocket_id);
  }
}

///// Send websocket message in all sessions which joined a specific room.
/////
///// `message` - The json message body to send
///// `room` - Connection IDs which should receive the message
///// `exclude_connection` - Dont send to user who initiated the api call, as that
/////                        would result in duplicate notification
//async fn send_message_in_room(
//  &self,
//  message: &str,
//  room: Option<HashSet<ConnectionId>>,
//  exclude_connection: Option<ConnectionId>,
//) -> Result<(), LemmyError> {
//  let mut session = self.inner()?.sessions.clone();
//  if let Some(room) = room {
//    // Note, this will ignore any errors, such as closed connections
//    join_all(
//      room
//        .into_iter()
//        .filter(|c| Some(c) != exclude_connection.as_ref())
//        .filter_map(|c| session.remove(&c))
//        .map(|mut s: Session| async move { s.text(message).await }),
//    )
//    .await;
//  }
//  Ok(())
//}
//}
