use crate::websocket::{
  handlers::{SessionInfo, WsMessage},
  structs::CaptchaItem,
};
use actix::{Actor, Context};
use lemmy_db_schema::newtypes::{CommunityId, LocalUserId, PostId};
use lemmy_utils::ConnectionId;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};

pub struct ChatServer {
  /// A map from generated random ID to session addr
  pub sessions: HashMap<ConnectionId, SessionInfo>,

  /// A map from post_id to set of connectionIDs
  pub post_rooms: HashMap<PostId, HashSet<ConnectionId>>,

  /// A map from community to set of connectionIDs
  pub community_rooms: HashMap<CommunityId, HashSet<ConnectionId>>,

  pub mod_rooms: HashMap<CommunityId, HashSet<ConnectionId>>,

  /// A map from user id to its connection ID for joined users. Remember a user can have multiple
  /// sessions (IE clients)
  pub(super) user_rooms: HashMap<LocalUserId, HashSet<ConnectionId>>,

  pub(super) rng: StdRng,

  /// A list of the current captchas
  pub(super) captchas: Vec<CaptchaItem>,
}

/// `ChatServer` is an actor. It maintains list of connection client session.
/// And manages available rooms. Peers send messages to other peers in same
/// room through `ChatServer`.
impl ChatServer {
  pub fn new() -> ChatServer {
    ChatServer {
      sessions: Default::default(),
      post_rooms: Default::default(),
      community_rooms: Default::default(),
      mod_rooms: Default::default(),
      user_rooms: Default::default(),
      rng: StdRng::from_entropy(),
      captchas: vec![],
    }
  }

  pub fn send_message(
    &self,
    connections: &HashSet<ConnectionId>,
    message: &str,
    exclude_connection: Option<ConnectionId>,
  ) {
    for id in connections
      .iter()
      .filter(|c| Some(*c) != exclude_connection.as_ref())
    {
      if let Some(session) = self.sessions.get(id) {
        session.addr.do_send(WsMessage(message.to_owned()));
      }
    }
  }
}

impl Default for ChatServer {
  fn default() -> Self {
    Self::new()
  }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
  /// We are going to use simple Context, we just need ability to communicate
  /// with other actors.
  type Context = Context<Self>;
}
