use crate::websocket::chat_server::ChatServer;
use actix::{Context, Handler, Message};
use lemmy_db_schema::newtypes::{CommunityId, LocalUserId, PostId};
use lemmy_utils::ConnectionId;
use std::collections::HashSet;

/// Joining a Post room
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinPostRoom {
  pub post_id: PostId,
  pub id: ConnectionId,
}

impl Handler<JoinPostRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinPostRoom, _: &mut Context<Self>) -> Self::Result {
    // remove session from all rooms
    for sessions in self.post_rooms.values_mut() {
      sessions.remove(&msg.id);
    }

    // Also leave all communities
    // This avoids double messages
    // TODO found a bug, whereby community messages like
    // delete and remove aren't sent, because
    // you left the community room
    for sessions in self.community_rooms.values_mut() {
      sessions.remove(&msg.id);
    }

    self
      .post_rooms
      .entry(msg.post_id)
      .or_insert_with(HashSet::new)
      .insert(msg.id);
  }
}

/// Joining a Community Room
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinCommunityRoom {
  pub community_id: CommunityId,
  pub id: ConnectionId,
}

impl Handler<JoinCommunityRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinCommunityRoom, _: &mut Context<Self>) -> Self::Result {
    // remove session from all rooms
    for sessions in self.community_rooms.values_mut() {
      sessions.remove(&msg.id);
    }

    // Also leave all post rooms
    // This avoids double messages
    for sessions in self.post_rooms.values_mut() {
      sessions.remove(&msg.id);
    }

    self
      .community_rooms
      .entry(msg.community_id)
      .or_insert_with(HashSet::new)
      .insert(msg.id);
  }
}

/// Joining a Mod room
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinModRoom {
  pub community_id: CommunityId,
  pub id: ConnectionId,
}

impl Handler<JoinModRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinModRoom, _: &mut Context<Self>) -> Self::Result {
    // remove session from all rooms
    for sessions in self.mod_rooms.values_mut() {
      sessions.remove(&msg.id);
    }

    self
      .mod_rooms
      .entry(msg.community_id)
      .or_insert_with(HashSet::new)
      .insert(msg.id);
  }
}

/// Joining a User room
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinUserRoom {
  pub user_id: LocalUserId,
  pub id: ConnectionId,
}

impl Handler<JoinUserRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinUserRoom, _: &mut Context<Self>) -> Self::Result {
    // remove session from all rooms
    for sessions in self.user_rooms.values_mut() {
      sessions.remove(&msg.id);
    }

    self
      .user_rooms
      .entry(msg.user_id)
      .or_insert_with(HashSet::new)
      .insert(msg.id);
  }
}
