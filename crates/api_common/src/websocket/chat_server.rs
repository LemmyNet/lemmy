use crate::{
  comment::CommentResponse,
  post::PostResponse,
  websocket::{serialize_websocket_message, structs::CaptchaItem, OperationType},
};
use actix_ws::Session;
use anyhow::Context as acontext;
use futures::future::join_all;
use lemmy_db_schema::newtypes::{CommunityId, LocalUserId, PostId};
use lemmy_utils::{error::LemmyError, location_info, ConnectionId};
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;
use std::{
  collections::{HashMap, HashSet},
  sync::{Mutex, MutexGuard},
};
use tracing::log::warn;

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session.
pub struct ChatServer {
  inner: Mutex<ChatServerInner>,
}

pub struct ChatServerInner {
  /// A map from generated random ID to session addr
  pub sessions: HashMap<ConnectionId, Session>,

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
  pub fn startup() -> ChatServer {
    ChatServer {
      inner: Mutex::new(ChatServerInner {
        sessions: Default::default(),
        post_rooms: Default::default(),
        community_rooms: Default::default(),
        mod_rooms: Default::default(),
        user_rooms: Default::default(),
        rng: StdRng::from_entropy(),
        captchas: vec![],
      }),
    }
  }

  pub fn join_community_room(
    &self,
    community_id: CommunityId,
    id: ConnectionId,
  ) -> Result<(), LemmyError> {
    let mut inner = self.inner()?;
    // remove session from all rooms
    for sessions in inner.community_rooms.values_mut() {
      sessions.remove(&id);
    }

    // Also leave all post rooms
    // This avoids double messages
    for sessions in inner.post_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if inner.community_rooms.get_mut(&community_id).is_none() {
      inner.community_rooms.insert(community_id, HashSet::new());
    }

    inner
      .community_rooms
      .get_mut(&community_id)
      .context(location_info!())?
      .insert(id);
    Ok(())
  }

  pub fn join_mod_room(
    &self,
    community_id: CommunityId,
    id: ConnectionId,
  ) -> Result<(), LemmyError> {
    let mut inner = self.inner()?;
    // remove session from all rooms
    for sessions in inner.mod_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if inner.mod_rooms.get_mut(&community_id).is_none() {
      inner.mod_rooms.insert(community_id, HashSet::new());
    }

    inner
      .mod_rooms
      .get_mut(&community_id)
      .context(location_info!())?
      .insert(id);
    Ok(())
  }

  pub fn join_post_room(&self, post_id: PostId, id: ConnectionId) -> Result<(), LemmyError> {
    let mut inner = self.inner()?;
    // remove session from all rooms
    for sessions in inner.post_rooms.values_mut() {
      sessions.remove(&id);
    }

    // Also leave all communities
    // This avoids double messages
    // TODO found a bug, whereby community messages like
    // delete and remove aren't sent, because
    // you left the community room
    for sessions in inner.community_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if inner.post_rooms.get_mut(&post_id).is_none() {
      inner.post_rooms.insert(post_id, HashSet::new());
    }

    inner
      .post_rooms
      .get_mut(&post_id)
      .context(location_info!())?
      .insert(id);

    Ok(())
  }

  pub fn join_user_room(&self, user_id: LocalUserId, id: ConnectionId) -> Result<(), LemmyError> {
    let mut inner = self.inner()?;
    // remove session from all rooms
    for sessions in inner.user_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if inner.user_rooms.get_mut(&user_id).is_none() {
      inner.user_rooms.insert(user_id, HashSet::new());
    }

    inner
      .user_rooms
      .get_mut(&user_id)
      .context(location_info!())?
      .insert(id);
    Ok(())
  }

  async fn send_post_room_message<OP, Response>(
    &self,
    op: &OP,
    response: &Response,
    post_id: PostId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
    Response: Serialize,
  {
    let msg = serialize_websocket_message(op, response)?;
    let room = self.inner()?.post_rooms.get(&post_id).cloned();
    self.send_message_in_room(&msg, room, websocket_id).await?;
    Ok(())
  }

  /// Send message to all users viewing the given community.
  pub async fn send_community_room_message<OP, Response>(
    &self,
    op: &OP,
    response: &Response,
    community_id: CommunityId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
    Response: Serialize,
  {
    let msg = serialize_websocket_message(op, response)?;
    let room = self.inner()?.community_rooms.get(&community_id).cloned();
    self.send_message_in_room(&msg, room, websocket_id).await?;
    Ok(())
  }

  /// Send message to mods of a given community. Set community_id = 0 to send to site admins.
  pub async fn send_mod_room_message<OP, Response>(
    &self,
    op: OP,
    response: &Response,
    community_id: CommunityId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
    Response: Serialize,
  {
    let msg = serialize_websocket_message(&op, response)?;
    let room = self.inner()?.mod_rooms.get(&community_id).cloned();
    self.send_message_in_room(&msg, room, websocket_id).await?;
    Ok(())
  }

  pub async fn send_all_message<OP, Response>(
    &self,
    op: OP,
    response: &Response,
    exclude_connection: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
    Response: Serialize,
  {
    let msg = &serialize_websocket_message(&op, response)?;
    let sessions = self.inner()?.sessions.clone();
    // Note, this will ignore any errors, such as closed connections
    join_all(
      sessions
        .into_iter()
        .filter(|(id, _)| Some(id) != exclude_connection.as_ref())
        .map(|(_, mut s): (_, Session)| async move { s.text(msg).await }),
    )
    .await;
    Ok(())
  }

  pub async fn send_user_room_message<OP, Response>(
    &self,
    op: &OP,
    response: &Response,
    recipient_id: LocalUserId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
    Response: Serialize,
  {
    let msg = serialize_websocket_message(op, response)?;
    let room = self.inner()?.user_rooms.get(&recipient_id).cloned();
    self.send_message_in_room(&msg, room, websocket_id).await?;
    Ok(())
  }

  pub async fn send_comment<OP>(
    &self,
    user_operation: &OP,
    comment: &CommentResponse,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
  {
    let mut comment_reply_sent = comment.clone();

    // Strip out my specific user info
    comment_reply_sent.comment_view.my_vote = None;

    // Send it to the post room
    let mut comment_post_sent = comment_reply_sent.clone();
    // Remove the recipients here to separate mentions / user messages from post or community comments
    comment_post_sent.recipient_ids = Vec::new();
    self
      .send_post_room_message(
        user_operation,
        &comment_post_sent,
        comment_post_sent.comment_view.post.id,
        websocket_id,
      )
      .await?;

    // Send it to the community too
    self
      .send_community_room_message(
        user_operation,
        &comment_post_sent,
        CommunityId(0),
        websocket_id,
      )
      .await?;
    self
      .send_community_room_message(
        user_operation,
        &comment_post_sent,
        comment.comment_view.community.id,
        websocket_id,
      )
      .await?;

    // Send it to the recipient(s) including the mentioned users
    for recipient_id in &comment_reply_sent.recipient_ids {
      self
        .send_user_room_message(
          user_operation,
          &comment_reply_sent,
          *recipient_id,
          websocket_id,
        )
        .await?;
    }

    Ok(())
  }

  pub async fn send_post<OP>(
    &self,
    user_operation: &OP,
    post_res: &PostResponse,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
  {
    let community_id = post_res.post_view.community.id;

    // Don't send my data with it
    let mut post_sent = post_res.clone();
    post_sent.post_view.my_vote = None;

    // Send it to /c/all and that community
    self
      .send_community_room_message(user_operation, &post_sent, CommunityId(0), websocket_id)
      .await?;
    self
      .send_community_room_message(user_operation, &post_sent, community_id, websocket_id)
      .await?;

    // Send it to the post room
    self
      .send_post_room_message(
        user_operation,
        &post_sent,
        post_res.post_view.post.id,
        websocket_id,
      )
      .await?;

    Ok(())
  }

  /// Send websocket message in all sessions which joined a specific room.
  ///
  /// `message` - The json message body to send
  /// `room` - Connection IDs which should receive the message
  /// `exclude_connection` - Dont send to user who initiated the api call, as that
  ///                        would result in duplicate notification
  async fn send_message_in_room(
    &self,
    message: &str,
    room: Option<HashSet<ConnectionId>>,
    exclude_connection: Option<ConnectionId>,
  ) -> Result<(), LemmyError> {
    let mut session = self.inner()?.sessions.clone();
    if let Some(room) = room {
      // Note, this will ignore any errors, such as closed connections
      join_all(
        room
          .into_iter()
          .filter(|c| Some(c) != exclude_connection.as_ref())
          .filter_map(|c| session.remove(&c))
          .map(|mut s: Session| async move { s.text(message).await }),
      )
      .await;
    }
    Ok(())
  }

  pub(in crate::websocket) fn inner(&self) -> Result<MutexGuard<'_, ChatServerInner>, LemmyError> {
    match self.inner.lock() {
      Ok(g) => Ok(g),
      Err(e) => {
        warn!("Failed to lock chatserver mutex: {}", e);
        Err(LemmyError::from_message("Failed to lock chatserver mutex"))
      }
    }
  }
}
