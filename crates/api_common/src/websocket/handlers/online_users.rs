use crate::websocket::chat_server::ChatServer;
use actix::{Context, Handler, Message};
use lemmy_db_schema::newtypes::{CommunityId, PostId};

/// Getting the number of online connections
#[derive(Message)]
#[rtype(usize)]
pub struct GetUsersOnline;

/// Handler for Disconnect message.
impl Handler<GetUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, _msg: GetUsersOnline, _: &mut Context<Self>) -> Self::Result {
    self.sessions.len()
  }
}

/// Getting the number of post users online
#[derive(Message)]
#[rtype(usize)]
pub struct GetPostUsersOnline {
  pub post_id: PostId,
}

/// Handler for Disconnect message.
impl Handler<GetPostUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: GetPostUsersOnline, _: &mut Context<Self>) -> Self::Result {
    self
      .post_rooms
      .get(&msg.post_id)
      .map_or(1, std::collections::HashSet::len)
  }
}

/// Getting the number of post users online
#[derive(Message)]
#[rtype(usize)]
pub struct GetCommunityUsersOnline {
  pub community_id: CommunityId,
}

/// Handler for Disconnect message.
impl Handler<GetCommunityUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: GetCommunityUsersOnline, _: &mut Context<Self>) -> Self::Result {
    self
      .community_rooms
      .get(&msg.community_id)
      .map_or(1, std::collections::HashSet::len)
  }
}
