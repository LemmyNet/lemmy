use crate::{
  messages::*,
  serialize_websocket_message,
  LemmyContext,
  OperationType,
  UserOperation,
  UserOperationCrud,
};
use actix::prelude::*;
use anyhow::Context as acontext;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use lemmy_api_common::{comment::*, post::*};
use lemmy_db_schema::{
  newtypes::{CommunityId, LocalUserId, PostId},
  source::secret::Secret,
};
use lemmy_utils::{
  error::LemmyError,
  location_info,
  rate_limit::RateLimit,
  settings::structs::Settings,
  ConnectionId,
  IpAddr,
};
use rand::rngs::ThreadRng;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;
use serde_json::Value;
use std::{
  collections::{HashMap, HashSet},
  future::Future,
  str::FromStr,
};
use tokio::macros::support::Pin;

type MessageHandlerType = fn(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: &str,
) -> Pin<Box<dyn Future<Output = Result<String, LemmyError>> + '_>>;

type MessageHandlerCrudType = fn(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperationCrud,
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
  pub(super) user_rooms: HashMap<LocalUserId, HashSet<ConnectionId>>,

  pub(super) rng: ThreadRng,

  /// The DB Pool
  pub(super) pool: Pool<ConnectionManager<PgConnection>>,

  /// The Settings
  pub(super) settings: Settings,

  /// The Secrets
  pub(super) secret: Secret,

  /// Rate limiting based on rate type and IP addr
  pub(super) rate_limiter: RateLimit,

  /// A list of the current captchas
  pub(super) captchas: Vec<CaptchaItem>,

  message_handler: MessageHandlerType,
  message_handler_crud: MessageHandlerCrudType,

  /// An HTTP Client
  client: ClientWithMiddleware,
}

pub struct SessionInfo {
  pub addr: Recipient<WsMessage>,
  pub ip: IpAddr,
}

/// `ChatServer` is an actor. It maintains list of connection client session.
/// And manages available rooms. Peers send messages to other peers in same
/// room through `ChatServer`.
impl ChatServer {
  #![allow(clippy::too_many_arguments)]
  pub fn startup(
    pool: Pool<ConnectionManager<PgConnection>>,
    rate_limiter: RateLimit,
    message_handler: MessageHandlerType,
    message_handler_crud: MessageHandlerCrudType,
    client: ClientWithMiddleware,
    settings: Settings,
    secret: Secret,
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
      message_handler_crud,
      client,
      settings,
      secret,
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

  pub fn join_user_room(
    &mut self,
    user_id: LocalUserId,
    id: ConnectionId,
  ) -> Result<(), LemmyError> {
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

  fn send_post_room_message<OP, Response>(
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

  pub fn send_community_room_message<OP, Response>(
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

  pub fn send_mod_room_message<OP, Response>(
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

  pub fn send_all_message<OP, Response>(
    &self,
    op: &OP,
    response: &Response,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    OP: OperationType + ToString,
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

  pub fn send_user_room_message<OP, Response>(
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

  pub fn send_comment<OP>(
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
    self.send_post_room_message(
      user_operation,
      &comment_post_sent,
      comment_post_sent.comment_view.post.id,
      websocket_id,
    )?;

    // Send it to the community too
    self.send_community_room_message(
      user_operation,
      &comment_post_sent,
      CommunityId(0),
      websocket_id,
    )?;
    self.send_community_room_message(
      user_operation,
      &comment_post_sent,
      comment.comment_view.community.id,
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

    Ok(())
  }

  pub fn send_post<OP>(
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
    self.send_community_room_message(user_operation, &post_sent, CommunityId(0), websocket_id)?;
    self.send_community_room_message(user_operation, &post_sent, community_id, websocket_id)?;

    // Send it to the post room
    self.send_post_room_message(
      user_operation,
      &post_sent,
      post_res.post_view.post.id,
      websocket_id,
    )?;

    Ok(())
  }

  fn sendit(&self, message: &str, id: ConnectionId) {
    if let Some(info) = self.sessions.get(&id) {
      info.addr.do_send(WsMessage(message.to_owned()));
    }
  }

  pub(super) fn parse_json_message(
    &mut self,
    msg: StandardMessage,
    ctx: &mut Context<Self>,
  ) -> impl Future<Output = Result<String, LemmyError>> {
    let rate_limiter = self.rate_limiter.clone();

    let ip: IpAddr = match self.sessions.get(&msg.id) {
      Some(info) => info.ip.to_owned(),
      None => IpAddr("blank_ip".to_string()),
    };

    let context = LemmyContext {
      pool: self.pool.clone(),
      chat_server: ctx.address(),
      client: self.client.to_owned(),
      settings: self.settings.to_owned(),
      secret: self.secret.to_owned(),
    };
    let message_handler_crud = self.message_handler_crud;
    let message_handler = self.message_handler;
    async move {
      let json: Value = serde_json::from_str(&msg.msg)?;
      let data = &json["data"].to_string();
      let op = &json["op"]
        .as_str()
        .ok_or_else(|| LemmyError::from_message("missing op"))?;

      // check if api call passes the rate limit, and generate future for later execution
      let (passed, fut) = if let Ok(user_operation_crud) = UserOperationCrud::from_str(op) {
        let passed = match user_operation_crud {
          UserOperationCrud::Register => rate_limiter.register().check(ip),
          UserOperationCrud::CreatePost => rate_limiter.post().check(ip),
          UserOperationCrud::CreateCommunity => rate_limiter.register().check(ip),
          UserOperationCrud::CreateComment => rate_limiter.comment().check(ip),
          _ => rate_limiter.message().check(ip),
        };
        let fut = (message_handler_crud)(context, msg.id, user_operation_crud, data);
        (passed, fut)
      } else {
        let user_operation = UserOperation::from_str(op)?;
        let passed = match user_operation {
          UserOperation::GetCaptcha => rate_limiter.post().check(ip),
          UserOperation::Search => rate_limiter.search().check(ip),
          _ => rate_limiter.message().check(ip),
        };
        let fut = (message_handler)(context, msg.id, user_operation, data);
        (passed, fut)
      };

      // if rate limit passed, execute api call future
      if passed {
        fut.await
      } else {
        // if rate limit was hit, respond with empty message
        Ok("".to_string())
      }
    }
  }
}
