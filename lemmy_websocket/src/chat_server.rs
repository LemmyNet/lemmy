use crate::{messages::*, serialize_websocket_message, LemmyContext, UserOperation};
use actix::prelude::*;
use anyhow::Context as acontext;
use background_jobs::QueueHandle;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use lemmy_rate_limit::RateLimit;
use lemmy_structs::{comment::*, post::*};
use lemmy_utils::{
  location_info,
  APIError,
  CommunityId,
  ConnectionId,
  IPAddr,
  LemmyError,
  PostId,
  UserId,
};
use rand::rngs::ThreadRng;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};
use tokio::macros::support::Pin;

type MessageHandlerType = fn(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: &str,
) -> Pin<Box<dyn Future<Output = Result<String, LemmyError>> + '_>>;

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session.
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
  pub(super) user_rooms: HashMap<UserId, HashSet<ConnectionId>>,

  pub(super) rng: ThreadRng,

  /// The DB Pool
  pub(super) pool: Pool<ConnectionManager<PgConnection>>,

  /// Rate limiting based on rate type and IP addr
  pub(super) rate_limiter: RateLimit,

  /// A list of the current captchas
  pub(super) captchas: Vec<CaptchaItem>,

  message_handler: MessageHandlerType,

  /// An HTTP Client
  client: Client,

  activity_queue: QueueHandle,
}

pub struct SessionInfo {
  pub addr: Recipient<WSMessage>,
  pub ip: IPAddr,
}

/// `ChatServer` is an actor. It maintains list of connection client session.
/// And manages available rooms. Peers send messages to other peers in same
/// room through `ChatServer`.
impl ChatServer {
  pub fn startup(
    pool: Pool<ConnectionManager<PgConnection>>,
    rate_limiter: RateLimit,
    message_handler: MessageHandlerType,
    client: Client,
    activity_queue: QueueHandle,
  ) -> ChatServer {
    ChatServer {
      sessions: HashMap::new(),
      post_rooms: HashMap::new(),
      community_rooms: HashMap::new(),
      mod_rooms: HashMap::new(),
      user_rooms: HashMap::new(),
      rng: rand::thread_rng(),
      pool,
      rate_limiter,
      captchas: Vec::new(),
      message_handler,
      client,
      activity_queue,
    }
  }

  pub fn join_community_room(
    &mut self,
    community_id: CommunityId,
    id: ConnectionId,
  ) -> Result<(), LemmyError> {
    // remove session from all rooms
    for sessions in self.community_rooms.values_mut() {
      sessions.remove(&id);
    }

    // Also leave all post rooms
    // This avoids double messages
    for sessions in self.post_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.community_rooms.get_mut(&community_id).is_none() {
      self.community_rooms.insert(community_id, HashSet::new());
    }

    self
      .community_rooms
      .get_mut(&community_id)
      .context(location_info!())?
      .insert(id);
    Ok(())
  }

  pub fn join_mod_room(
    &mut self,
    community_id: CommunityId,
    id: ConnectionId,
  ) -> Result<(), LemmyError> {
    // remove session from all rooms
    for sessions in self.mod_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.mod_rooms.get_mut(&community_id).is_none() {
      self.mod_rooms.insert(community_id, HashSet::new());
    }

    self
      .mod_rooms
      .get_mut(&community_id)
      .context(location_info!())?
      .insert(id);
    Ok(())
  }

  pub fn join_post_room(&mut self, post_id: PostId, id: ConnectionId) -> Result<(), LemmyError> {
    // remove session from all rooms
    for sessions in self.post_rooms.values_mut() {
      sessions.remove(&id);
    }

    // Also leave all communities
    // This avoids double messages
    // TODO found a bug, whereby community messages like
    // delete and remove aren't sent, because
    // you left the community room
    for sessions in self.community_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.post_rooms.get_mut(&post_id).is_none() {
      self.post_rooms.insert(post_id, HashSet::new());
    }

    self
      .post_rooms
      .get_mut(&post_id)
      .context(location_info!())?
      .insert(id);

    Ok(())
  }

  pub fn join_user_room(&mut self, user_id: UserId, id: ConnectionId) -> Result<(), LemmyError> {
    // remove session from all rooms
    for sessions in self.user_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.user_rooms.get_mut(&user_id).is_none() {
      self.user_rooms.insert(user_id, HashSet::new());
    }

    self
      .user_rooms
      .get_mut(&user_id)
      .context(location_info!())?
      .insert(id);

    Ok(())
  }

  fn send_post_room_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    post_id: PostId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Response: Serialize,
  {
    let res_str = &serialize_websocket_message(op, response)?;
    if let Some(sessions) = self.post_rooms.get(&post_id) {
      for id in sessions {
        if let Some(my_id) = websocket_id {
          if *id == my_id {
            continue;
          }
        }
        self.sendit(res_str, *id);
      }
    }
    Ok(())
  }

  pub fn send_community_room_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    community_id: CommunityId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Response: Serialize,
  {
    let res_str = &serialize_websocket_message(op, response)?;
    if let Some(sessions) = self.community_rooms.get(&community_id) {
      for id in sessions {
        if let Some(my_id) = websocket_id {
          if *id == my_id {
            continue;
          }
        }
        self.sendit(res_str, *id);
      }
    }
    Ok(())
  }

  pub fn send_mod_room_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    community_id: CommunityId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Response: Serialize,
  {
    let res_str = &serialize_websocket_message(op, response)?;
    if let Some(sessions) = self.mod_rooms.get(&community_id) {
      for id in sessions {
        if let Some(my_id) = websocket_id {
          if *id == my_id {
            continue;
          }
        }
        self.sendit(res_str, *id);
      }
    }
    Ok(())
  }

  pub fn send_all_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Response: Serialize,
  {
    let res_str = &serialize_websocket_message(op, response)?;
    for id in self.sessions.keys() {
      if let Some(my_id) = websocket_id {
        if *id == my_id {
          continue;
        }
      }
      self.sendit(res_str, *id);
    }
    Ok(())
  }

  pub fn send_user_room_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    recipient_id: UserId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Response: Serialize,
  {
    let res_str = &serialize_websocket_message(op, response)?;
    if let Some(sessions) = self.user_rooms.get(&recipient_id) {
      for id in sessions {
        if let Some(my_id) = websocket_id {
          if *id == my_id {
            continue;
          }
        }
        self.sendit(res_str, *id);
      }
    }
    Ok(())
  }

  pub fn send_comment(
    &self,
    user_operation: &UserOperation,
    comment: &CommentResponse,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError> {
    let mut comment_reply_sent = comment.clone();
    comment_reply_sent.comment.my_vote = None;
    comment_reply_sent.comment.user_id = None;

    let mut comment_post_sent = comment_reply_sent.clone();
    comment_post_sent.recipient_ids = Vec::new();

    // Send it to the post room
    self.send_post_room_message(
      user_operation,
      &comment_post_sent,
      comment_post_sent.comment.post_id,
      websocket_id,
    )?;

    // Send it to the recipient(s) including the mentioned users
    for recipient_id in &comment_reply_sent.recipient_ids {
      self.send_user_room_message(
        user_operation,
        &comment_reply_sent,
        *recipient_id,
        websocket_id,
      )?;
    }

    // Send it to the community too
    self.send_community_room_message(user_operation, &comment_post_sent, 0, websocket_id)?;
    self.send_community_room_message(
      user_operation,
      &comment_post_sent,
      comment.comment.community_id,
      websocket_id,
    )?;

    Ok(())
  }

  pub fn send_post(
    &self,
    user_operation: &UserOperation,
    post: &PostResponse,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError> {
    let community_id = post.post.community_id;

    // Don't send my data with it
    let mut post_sent = post.clone();
    post_sent.post.my_vote = None;
    post_sent.post.user_id = None;

    // Send it to /c/all and that community
    self.send_community_room_message(user_operation, &post_sent, 0, websocket_id)?;
    self.send_community_room_message(user_operation, &post_sent, community_id, websocket_id)?;

    // Send it to the post room
    self.send_post_room_message(user_operation, &post_sent, post.post.id, websocket_id)?;

    Ok(())
  }

  fn sendit(&self, message: &str, id: ConnectionId) {
    if let Some(info) = self.sessions.get(&id) {
      let _ = info.addr.do_send(WSMessage(message.to_owned()));
    }
  }

  pub(super) fn parse_json_message(
    &mut self,
    msg: StandardMessage,
    ctx: &mut Context<Self>,
  ) -> impl Future<Output = Result<String, LemmyError>> {
    let rate_limiter = self.rate_limiter.clone();

    let ip: IPAddr = match self.sessions.get(&msg.id) {
      Some(info) => info.ip.to_owned(),
      None => "blank_ip".to_string(),
    };

    let context = LemmyContext {
      pool: self.pool.clone(),
      chat_server: ctx.address(),
      client: self.client.to_owned(),
      activity_queue: self.activity_queue.to_owned(),
    };
    let message_handler = self.message_handler;
    async move {
      let json: Value = serde_json::from_str(&msg.msg)?;
      let data = &json["data"].to_string();
      let op = &json["op"].as_str().ok_or(APIError {
        message: "Unknown op type".to_string(),
      })?;

      let user_operation = UserOperation::from_str(&op)?;
      let fut = (message_handler)(context, msg.id, user_operation.clone(), data);
      match user_operation {
        UserOperation::Register => rate_limiter.register().wrap(ip, fut).await,
        UserOperation::CreatePost => rate_limiter.post().wrap(ip, fut).await,
        UserOperation::CreateCommunity => rate_limiter.register().wrap(ip, fut).await,
        _ => rate_limiter.message().wrap(ip, fut).await,
      }
    }
  }
}
