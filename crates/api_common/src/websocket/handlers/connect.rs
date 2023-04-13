use crate::websocket::{
  chat_server::ChatServer,
  handlers::{SessionInfo, WsMessage},
};
use actix::{Context, Handler, Message, Recipient};
use lemmy_utils::ConnectionId;
use rand::Rng;

/// New chat session is created
#[derive(Message)]
#[rtype(ConnectionId)]
pub struct Connect {
  pub addr: Recipient<WsMessage>,
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
  type Result = ConnectionId;

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    // register session with random id
    let id = self.rng.gen::<usize>();
    let session = SessionInfo { addr: msg.addr };
    self.sessions.insert(id, session);

    // send id back
    id
  }
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub id: ConnectionId,
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
    // remove address
    if self.sessions.remove(&msg.id).is_some() {
      // remove session from all rooms
      for sessions in self.user_rooms.values_mut() {
        sessions.remove(&msg.id);
      }
      for sessions in self.post_rooms.values_mut() {
        sessions.remove(&msg.id);
      }
      for sessions in self.community_rooms.values_mut() {
        sessions.remove(&msg.id);
      }
      for sessions in self.mod_rooms.values_mut() {
        sessions.remove(&msg.id);
      }
    }
  }
}
