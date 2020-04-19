//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use super::*;
use crate::api::comment::*;
use crate::api::community::*;
use crate::api::post::*;
use crate::api::site::*;
use crate::api::user::*;
use crate::api::*;
use crate::rate_limit::{rate_limiter::RateLimiter, RateLimitInfo};
use crate::websocket::UserOperation;
use crate::{CommunityId, ConnectionId, IPAddr, PostId, UserId};

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
#[rtype(String)]
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
  pub my_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUserRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub recipient_id: UserId,
  pub my_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommunityRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub community_id: CommunityId,
  pub my_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPost {
  pub op: UserOperation,
  pub post: PostResponse,
  pub my_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendComment {
  pub op: UserOperation,
  pub comment: CommentResponse,
  pub my_id: Option<ConnectionId>,
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

pub struct SessionInfo {
  pub addr: Recipient<WSMessage>,
  pub ip: IPAddr,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session.
pub struct ChatServer {
  /// A map from generated random ID to session addr
  pub sessions: HashMap<ConnectionId, SessionInfo>,

  /// A map from post_id to set of connectionIDs
  pub post_rooms: HashMap<PostId, HashSet<ConnectionId>>,

  /// A map from community to set of connectionIDs
  pub community_rooms: HashMap<CommunityId, HashSet<ConnectionId>>,

  /// A map from user id to its connection ID for joined users. Remember a user can have multiple
  /// sessions (IE clients)
  user_rooms: HashMap<UserId, HashSet<ConnectionId>>,

  rng: ThreadRng,

  /// The DB Pool
  pool: Pool<ConnectionManager<PgConnection>>,

  /// Rate limiting based on rate type and IP addr
  rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl ChatServer {
  pub fn startup(
    pool: Pool<ConnectionManager<PgConnection>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
  ) -> ChatServer {
    ChatServer {
      sessions: HashMap::new(),
      post_rooms: HashMap::new(),
      community_rooms: HashMap::new(),
      user_rooms: HashMap::new(),
      rng: rand::thread_rng(),
      pool,
      rate_limiter,
    }
  }

  pub fn join_community_room(&mut self, community_id: CommunityId, id: ConnectionId) {
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
      .unwrap()
      .insert(id);
  }

  pub fn join_post_room(&mut self, post_id: PostId, id: ConnectionId) {
    // remove session from all rooms
    for sessions in self.post_rooms.values_mut() {
      sessions.remove(&id);
    }

    // Also leave all communities
    // This avoids double messages
    for sessions in self.community_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.post_rooms.get_mut(&post_id).is_none() {
      self.post_rooms.insert(post_id, HashSet::new());
    }

    self.post_rooms.get_mut(&post_id).unwrap().insert(id);
  }

  pub fn join_user_room(&mut self, user_id: UserId, id: ConnectionId) {
    // remove session from all rooms
    for sessions in self.user_rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.user_rooms.get_mut(&user_id).is_none() {
      self.user_rooms.insert(user_id, HashSet::new());
    }

    self.user_rooms.get_mut(&user_id).unwrap().insert(id);
  }

  fn send_post_room_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    post_id: PostId,
    my_id: Option<ConnectionId>,
  ) -> Result<(), Error>
  where
    Response: Serialize,
  {
    let res_str = &to_json_string(op, response)?;
    if let Some(sessions) = self.post_rooms.get(&post_id) {
      for id in sessions {
        if let Some(my_id) = my_id {
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
    my_id: Option<ConnectionId>,
  ) -> Result<(), Error>
  where
    Response: Serialize,
  {
    let res_str = &to_json_string(op, response)?;
    if let Some(sessions) = self.community_rooms.get(&community_id) {
      for id in sessions {
        if let Some(my_id) = my_id {
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
    my_id: Option<ConnectionId>,
  ) -> Result<(), Error>
  where
    Response: Serialize,
  {
    let res_str = &to_json_string(op, response)?;
    for id in self.sessions.keys() {
      if let Some(my_id) = my_id {
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
    my_id: Option<ConnectionId>,
  ) -> Result<(), Error>
  where
    Response: Serialize,
  {
    let res_str = &to_json_string(op, response)?;
    if let Some(sessions) = self.user_rooms.get(&recipient_id) {
      for id in sessions {
        if let Some(my_id) = my_id {
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
    my_id: Option<ConnectionId>,
  ) -> Result<(), Error> {
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
      my_id,
    )?;

    // Send it to the recipient(s) including the mentioned users
    for recipient_id in &comment_reply_sent.recipient_ids {
      self.send_user_room_message(user_operation, &comment_reply_sent, *recipient_id, my_id)?;
    }

    // Send it to the community too
    self.send_community_room_message(user_operation, &comment_post_sent, 0, my_id)?;
    self.send_community_room_message(
      user_operation,
      &comment_post_sent,
      comment.comment.community_id,
      my_id,
    )?;

    Ok(())
  }

  pub fn send_post(
    &self,
    user_operation: &UserOperation,
    post: &PostResponse,
    my_id: Option<ConnectionId>,
  ) -> Result<(), Error> {
    let community_id = post.post.community_id;

    // Don't send my data with it
    let mut post_sent = post.clone();
    post_sent.post.my_vote = None;
    post_sent.post.user_id = None;

    // Send it to /c/all and that community
    self.send_community_room_message(user_operation, &post_sent, 0, my_id)?;
    self.send_community_room_message(user_operation, &post_sent, community_id, my_id)?;

    // Send it to the post room
    self.send_post_room_message(user_operation, &post_sent, post.post.id, my_id)?;

    Ok(())
  }

  fn sendit(&self, message: &str, id: ConnectionId) {
    if let Some(info) = self.sessions.get(&id) {
      let _ = info.addr.do_send(WSMessage(message.to_owned()));
    }
  }

  fn do_user_operation<'a, Data, Response>(
    &self,
    id: ConnectionId,
    ip: IPAddr,
    op: UserOperation,
    data: &str,
    ctx: &mut Context<Self>,
  ) -> Result<String, Error>
  where
    for<'de> Data: Deserialize<'de> + 'a,
    Response: Serialize,
    Oper<Data>: Perform<Response>,
  {
    let parsed_data: Data = serde_json::from_str(data)?;

    let ws_info = WebsocketInfo {
      chatserver: ctx.address(),
      id: Some(id),
    };

    let rate_limit_info = RateLimitInfo {
      rate_limiter: self.rate_limiter.clone(),
      ip,
    };

    let new_pool = self.pool.clone();
    let res = Oper::new(parsed_data).perform(new_pool, Some(ws_info), Some(rate_limit_info))?;
    to_json_string(&op, &res)
  }

  fn parse_json_message(
    &mut self,
    msg: StandardMessage,
    ctx: &mut Context<Self>,
  ) -> Result<String, Error> {
    let json: Value = serde_json::from_str(&msg.msg)?;
    let data = &json["data"].to_string();
    let op = &json["op"].as_str().ok_or(APIError {
      message: "Unknown op type".to_string(),
    })?;

    let user_operation: UserOperation = UserOperation::from_str(&op)?;

    let ip: IPAddr = match self.sessions.get(&msg.id) {
      Some(info) => info.ip.to_owned(),
      None => "blank_ip".to_string(),
    };

    match user_operation {
      // User ops
      UserOperation::Login => {
        self.do_user_operation::<Login, LoginResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::Register => {
        self.do_user_operation::<Register, LoginResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::GetUserDetails => self
        .do_user_operation::<GetUserDetails, GetUserDetailsResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::GetReplies => self.do_user_operation::<GetReplies, GetRepliesResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::AddAdmin => {
        self.do_user_operation::<AddAdmin, AddAdminResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::BanUser => {
        self.do_user_operation::<BanUser, BanUserResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::GetUserMentions => self
        .do_user_operation::<GetUserMentions, GetUserMentionsResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::EditUserMention => self
        .do_user_operation::<EditUserMention, UserMentionResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::MarkAllAsRead => self.do_user_operation::<MarkAllAsRead, GetRepliesResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::DeleteAccount => self.do_user_operation::<DeleteAccount, LoginResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::PasswordReset => self
        .do_user_operation::<PasswordReset, PasswordResetResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::PasswordChange => self.do_user_operation::<PasswordChange, LoginResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::CreatePrivateMessage => self
        .do_user_operation::<CreatePrivateMessage, PrivateMessageResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::EditPrivateMessage => self
        .do_user_operation::<EditPrivateMessage, PrivateMessageResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::GetPrivateMessages => self
        .do_user_operation::<GetPrivateMessages, PrivateMessagesResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::UserJoin => {
        self.do_user_operation::<UserJoin, UserJoinResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::SaveUserSettings => self.do_user_operation::<SaveUserSettings, LoginResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),

      // Site ops
      UserOperation::GetModlog => self.do_user_operation::<GetModlog, GetModlogResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::CreateSite => {
        self.do_user_operation::<CreateSite, SiteResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::EditSite => {
        self.do_user_operation::<EditSite, SiteResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::GetSite => {
        self.do_user_operation::<GetSite, GetSiteResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::GetSiteConfig => self
        .do_user_operation::<GetSiteConfig, GetSiteConfigResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::SaveSiteConfig => self
        .do_user_operation::<SaveSiteConfig, GetSiteConfigResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::Search => {
        self.do_user_operation::<Search, SearchResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::TransferCommunity => self
        .do_user_operation::<TransferCommunity, GetCommunityResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::TransferSite => self.do_user_operation::<TransferSite, GetSiteResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::ListCategories => self
        .do_user_operation::<ListCategories, ListCategoriesResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),

      // Community ops
      UserOperation::GetCommunity => self.do_user_operation::<GetCommunity, GetCommunityResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::ListCommunities => self
        .do_user_operation::<ListCommunities, ListCommunitiesResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::CreateCommunity => self
        .do_user_operation::<CreateCommunity, CommunityResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::EditCommunity => self.do_user_operation::<EditCommunity, CommunityResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::FollowCommunity => self
        .do_user_operation::<FollowCommunity, CommunityResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::GetFollowedCommunities => self
        .do_user_operation::<GetFollowedCommunities, GetFollowedCommunitiesResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::BanFromCommunity => self
        .do_user_operation::<BanFromCommunity, BanFromCommunityResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
      UserOperation::AddModToCommunity => self
        .do_user_operation::<AddModToCommunity, AddModToCommunityResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),

      // Post ops
      UserOperation::CreatePost => {
        self.do_user_operation::<CreatePost, PostResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::GetPost => {
        self.do_user_operation::<GetPost, GetPostResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::GetPosts => {
        self.do_user_operation::<GetPosts, GetPostsResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::EditPost => {
        self.do_user_operation::<EditPost, PostResponse>(msg.id, ip, user_operation, data, ctx)
      }
      UserOperation::CreatePostLike => self.do_user_operation::<CreatePostLike, PostResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::SavePost => {
        self.do_user_operation::<SavePost, PostResponse>(msg.id, ip, user_operation, data, ctx)
      }

      // Comment ops
      UserOperation::CreateComment => self.do_user_operation::<CreateComment, CommentResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::EditComment => self.do_user_operation::<EditComment, CommentResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::SaveComment => self.do_user_operation::<SaveComment, CommentResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::GetComments => self.do_user_operation::<GetComments, GetCommentsResponse>(
        msg.id,
        ip,
        user_operation,
        data,
        ctx,
      ),
      UserOperation::CreateCommentLike => self
        .do_user_operation::<CreateCommentLike, CommentResponse>(
          msg.id,
          ip,
          user_operation,
          data,
          ctx,
        ),
    }
  }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
  /// We are going to use simple Context, we just need ability to communicate
  /// with other actors.
  type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
    // register session with random id
    let id = self.rng.gen::<usize>();
    info!("{} joined", &msg.ip);

    self.sessions.insert(
      id,
      SessionInfo {
        addr: msg.addr,
        ip: msg.ip.to_owned(),
      },
    );

    id
  }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    // Remove connections from sessions and all 3 scopes
    if self.sessions.remove(&msg.id).is_some() {
      for sessions in self.user_rooms.values_mut() {
        sessions.remove(&msg.id);
      }

      for sessions in self.post_rooms.values_mut() {
        sessions.remove(&msg.id);
      }

      for sessions in self.community_rooms.values_mut() {
        sessions.remove(&msg.id);
      }
    }
  }
}

/// Handler for Message message.
impl Handler<StandardMessage> for ChatServer {
  type Result = MessageResult<StandardMessage>;

  fn handle(&mut self, msg: StandardMessage, ctx: &mut Context<Self>) -> Self::Result {
    match self.parse_json_message(msg, ctx) {
      Ok(m) => {
        info!("Message Sent: {}", m);
        MessageResult(m)
      }
      Err(e) => {
        error!("Error during message handling {}", e);
        MessageResult(e.to_string())
      }
    }
  }
}

impl<Response> Handler<SendAllMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendAllMessage<Response>, _: &mut Context<Self>) {
    self
      .send_all_message(&msg.op, &msg.response, msg.my_id)
      .unwrap();
  }
}

impl<Response> Handler<SendUserRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendUserRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_user_room_message(&msg.op, &msg.response, msg.recipient_id, msg.my_id)
      .unwrap();
  }
}

impl<Response> Handler<SendCommunityRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendCommunityRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_community_room_message(&msg.op, &msg.response, msg.community_id, msg.my_id)
      .unwrap();
  }
}

impl Handler<SendPost> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendPost, _: &mut Context<Self>) {
    self.send_post(&msg.op, &msg.post, msg.my_id).unwrap();
  }
}

impl Handler<SendComment> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendComment, _: &mut Context<Self>) {
    self.send_comment(&msg.op, &msg.comment, msg.my_id).unwrap();
  }
}

impl Handler<JoinUserRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinUserRoom, _: &mut Context<Self>) {
    self.join_user_room(msg.user_id, msg.id);
  }
}

impl Handler<JoinCommunityRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinCommunityRoom, _: &mut Context<Self>) {
    self.join_community_room(msg.community_id, msg.id);
  }
}

impl Handler<JoinPostRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinPostRoom, _: &mut Context<Self>) {
    self.join_post_room(msg.post_id, msg.id);
  }
}

impl Handler<GetUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, _msg: GetUsersOnline, _: &mut Context<Self>) -> Self::Result {
    self.sessions.len()
  }
}

impl Handler<GetPostUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: GetPostUsersOnline, _: &mut Context<Self>) -> Self::Result {
    if let Some(users) = self.post_rooms.get(&msg.post_id) {
      users.len()
    } else {
      0
    }
  }
}

impl Handler<GetCommunityUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: GetCommunityUsersOnline, _: &mut Context<Self>) -> Self::Result {
    if let Some(users) = self.community_rooms.get(&msg.community_id) {
      users.len()
    } else {
      0
    }
  }
}

#[derive(Serialize)]
struct WebsocketResponse<T> {
  op: String,
  data: T,
}

fn to_json_string<Response>(op: &UserOperation, data: &Response) -> Result<String, Error>
where
  Response: Serialize,
{
  let response = WebsocketResponse {
    op: op.to_string(),
    data,
  };
  Ok(serde_json::to_string(&response)?)
}
