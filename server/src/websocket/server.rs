//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use super::*;
use crate::{
  api::{comment::*, community::*, post::*, site::*, user::*, *},
  rate_limit::RateLimit,
  websocket::UserOperation,
  CommunityId,
  ConnectionId,
  IPAddr,
  LemmyContext,
  LemmyError,
  PostId,
  UserId,
};
use actix_web::{client::Client, web};
use anyhow::Context as acontext;
use lemmy_db::naive_now;
use lemmy_utils::location_info;

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
#[rtype(result = "Result<String, std::convert::Infallible>")]
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
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUserRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub recipient_id: UserId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommunityRoomMessage<Response> {
  pub op: UserOperation,
  pub response: Response,
  pub community_id: CommunityId,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPost {
  pub op: UserOperation,
  pub post: PostResponse,
  pub websocket_id: Option<ConnectionId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendComment {
  pub op: UserOperation,
  pub comment: CommentResponse,
  pub websocket_id: Option<ConnectionId>,
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

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CaptchaItem {
  pub uuid: String,
  pub answer: String,
  pub expires: chrono::NaiveDateTime,
}

#[derive(Message)]
#[rtype(bool)]
pub struct CheckCaptcha {
  pub uuid: String,
  pub answer: String,
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
  rate_limiter: RateLimit,

  /// A list of the current captchas
  captchas: Vec<CaptchaItem>,

  /// An HTTP Client
  client: Client,
}

impl ChatServer {
  pub fn startup(
    pool: Pool<ConnectionManager<PgConnection>>,
    rate_limiter: RateLimit,
    client: Client,
  ) -> ChatServer {
    ChatServer {
      sessions: HashMap::new(),
      post_rooms: HashMap::new(),
      community_rooms: HashMap::new(),
      user_rooms: HashMap::new(),
      rng: rand::thread_rng(),
      pool,
      rate_limiter,
      captchas: Vec::new(),
      client,
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
    let res_str = &to_json_string(op, response)?;
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
    let res_str = &to_json_string(op, response)?;
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

  pub fn send_all_message<Response>(
    &self,
    op: &UserOperation,
    response: &Response,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Response: Serialize,
  {
    let res_str = &to_json_string(op, response)?;
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
    let res_str = &to_json_string(op, response)?;
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

  fn parse_json_message(
    &mut self,
    msg: StandardMessage,
    ctx: &mut Context<Self>,
  ) -> impl Future<Output = Result<String, LemmyError>> {
    let addr = ctx.address();
    let pool = self.pool.clone();
    let rate_limiter = self.rate_limiter.clone();

    let ip: IPAddr = match self.sessions.get(&msg.id) {
      Some(info) => info.ip.to_owned(),
      None => "blank_ip".to_string(),
    };

    let client = self.client.clone();
    async move {
      let msg = msg;
      let json: Value = serde_json::from_str(&msg.msg)?;
      let data = &json["data"].to_string();
      let op = &json["op"].as_str().ok_or(APIError {
        message: "Unknown op type".to_string(),
      })?;

      let user_operation: UserOperation = UserOperation::from_str(&op)?;

      let context = LemmyContext {
        pool,
        chat_server: addr,
        client,
      };
      let args = Args {
        context,
        rate_limiter,
        id: msg.id,
        ip,
        op: user_operation.clone(),
        data,
      };

      match user_operation {
        // User ops
        UserOperation::Login => do_user_operation::<Login>(args).await,
        UserOperation::Register => do_user_operation::<Register>(args).await,
        UserOperation::GetCaptcha => do_user_operation::<GetCaptcha>(args).await,
        UserOperation::GetUserDetails => do_user_operation::<GetUserDetails>(args).await,
        UserOperation::GetReplies => do_user_operation::<GetReplies>(args).await,
        UserOperation::AddAdmin => do_user_operation::<AddAdmin>(args).await,
        UserOperation::BanUser => do_user_operation::<BanUser>(args).await,
        UserOperation::GetUserMentions => do_user_operation::<GetUserMentions>(args).await,
        UserOperation::MarkUserMentionAsRead => {
          do_user_operation::<MarkUserMentionAsRead>(args).await
        }
        UserOperation::MarkAllAsRead => do_user_operation::<MarkAllAsRead>(args).await,
        UserOperation::DeleteAccount => do_user_operation::<DeleteAccount>(args).await,
        UserOperation::PasswordReset => do_user_operation::<PasswordReset>(args).await,
        UserOperation::PasswordChange => do_user_operation::<PasswordChange>(args).await,
        UserOperation::UserJoin => do_user_operation::<UserJoin>(args).await,
        UserOperation::SaveUserSettings => do_user_operation::<SaveUserSettings>(args).await,

        // Private Message ops
        UserOperation::CreatePrivateMessage => {
          do_user_operation::<CreatePrivateMessage>(args).await
        }
        UserOperation::EditPrivateMessage => do_user_operation::<EditPrivateMessage>(args).await,
        UserOperation::DeletePrivateMessage => {
          do_user_operation::<DeletePrivateMessage>(args).await
        }
        UserOperation::MarkPrivateMessageAsRead => {
          do_user_operation::<MarkPrivateMessageAsRead>(args).await
        }
        UserOperation::GetPrivateMessages => do_user_operation::<GetPrivateMessages>(args).await,

        // Site ops
        UserOperation::GetModlog => do_user_operation::<GetModlog>(args).await,
        UserOperation::CreateSite => do_user_operation::<CreateSite>(args).await,
        UserOperation::EditSite => do_user_operation::<EditSite>(args).await,
        UserOperation::GetSite => do_user_operation::<GetSite>(args).await,
        UserOperation::GetSiteConfig => do_user_operation::<GetSiteConfig>(args).await,
        UserOperation::SaveSiteConfig => do_user_operation::<SaveSiteConfig>(args).await,
        UserOperation::Search => do_user_operation::<Search>(args).await,
        UserOperation::TransferCommunity => do_user_operation::<TransferCommunity>(args).await,
        UserOperation::TransferSite => do_user_operation::<TransferSite>(args).await,
        UserOperation::ListCategories => do_user_operation::<ListCategories>(args).await,

        // Community ops
        UserOperation::GetCommunity => do_user_operation::<GetCommunity>(args).await,
        UserOperation::ListCommunities => do_user_operation::<ListCommunities>(args).await,
        UserOperation::CreateCommunity => do_user_operation::<CreateCommunity>(args).await,
        UserOperation::EditCommunity => do_user_operation::<EditCommunity>(args).await,
        UserOperation::DeleteCommunity => do_user_operation::<DeleteCommunity>(args).await,
        UserOperation::RemoveCommunity => do_user_operation::<RemoveCommunity>(args).await,
        UserOperation::FollowCommunity => do_user_operation::<FollowCommunity>(args).await,
        UserOperation::GetFollowedCommunities => {
          do_user_operation::<GetFollowedCommunities>(args).await
        }
        UserOperation::BanFromCommunity => do_user_operation::<BanFromCommunity>(args).await,
        UserOperation::AddModToCommunity => do_user_operation::<AddModToCommunity>(args).await,

        // Post ops
        UserOperation::CreatePost => do_user_operation::<CreatePost>(args).await,
        UserOperation::GetPost => do_user_operation::<GetPost>(args).await,
        UserOperation::GetPosts => do_user_operation::<GetPosts>(args).await,
        UserOperation::EditPost => do_user_operation::<EditPost>(args).await,
        UserOperation::DeletePost => do_user_operation::<DeletePost>(args).await,
        UserOperation::RemovePost => do_user_operation::<RemovePost>(args).await,
        UserOperation::LockPost => do_user_operation::<LockPost>(args).await,
        UserOperation::StickyPost => do_user_operation::<StickyPost>(args).await,
        UserOperation::CreatePostLike => do_user_operation::<CreatePostLike>(args).await,
        UserOperation::SavePost => do_user_operation::<SavePost>(args).await,

        // Comment ops
        UserOperation::CreateComment => do_user_operation::<CreateComment>(args).await,
        UserOperation::EditComment => do_user_operation::<EditComment>(args).await,
        UserOperation::DeleteComment => do_user_operation::<DeleteComment>(args).await,
        UserOperation::RemoveComment => do_user_operation::<RemoveComment>(args).await,
        UserOperation::MarkCommentAsRead => do_user_operation::<MarkCommentAsRead>(args).await,
        UserOperation::SaveComment => do_user_operation::<SaveComment>(args).await,
        UserOperation::GetComments => do_user_operation::<GetComments>(args).await,
        UserOperation::CreateCommentLike => do_user_operation::<CreateCommentLike>(args).await,
      }
    }
  }
}

struct Args<'a> {
  context: LemmyContext,
  rate_limiter: RateLimit,
  id: ConnectionId,
  ip: IPAddr,
  op: UserOperation,
  data: &'a str,
}

async fn do_user_operation<'a, 'b, Data>(args: Args<'b>) -> Result<String, LemmyError>
where
  for<'de> Data: Deserialize<'de> + 'a,
  Data: Perform,
{
  let Args {
    context,
    rate_limiter,
    id,
    ip,
    op,
    data,
  } = args;

  let data = data.to_string();
  let op2 = op.clone();

  let fut = async move {
    let parsed_data: Data = serde_json::from_str(&data)?;
    let res = parsed_data
      .perform(&web::Data::new(context), Some(id))
      .await?;
    to_json_string(&op, &res)
  };

  match op2 {
    UserOperation::Register => rate_limiter.register().wrap(ip, fut).await,
    UserOperation::CreatePost => rate_limiter.post().wrap(ip, fut).await,
    UserOperation::CreateCommunity => rate_limiter.register().wrap(ip, fut).await,
    _ => rate_limiter.message().wrap(ip, fut).await,
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
        ip: msg.ip,
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
  type Result = ResponseFuture<Result<String, std::convert::Infallible>>;

  fn handle(&mut self, msg: StandardMessage, ctx: &mut Context<Self>) -> Self::Result {
    let fut = self.parse_json_message(msg, ctx);
    Box::pin(async move {
      match fut.await {
        Ok(m) => {
          // info!("Message Sent: {}", m);
          Ok(m)
        }
        Err(e) => {
          error!("Error during message handling {}", e);
          Ok(e.to_string())
        }
      }
    })
  }
}

impl<Response> Handler<SendAllMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendAllMessage<Response>, _: &mut Context<Self>) {
    self
      .send_all_message(&msg.op, &msg.response, msg.websocket_id)
      .ok();
  }
}

impl<Response> Handler<SendUserRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendUserRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_user_room_message(&msg.op, &msg.response, msg.recipient_id, msg.websocket_id)
      .ok();
  }
}

impl<Response> Handler<SendCommunityRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendCommunityRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_community_room_message(&msg.op, &msg.response, msg.community_id, msg.websocket_id)
      .ok();
  }
}

impl Handler<SendPost> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendPost, _: &mut Context<Self>) {
    self.send_post(&msg.op, &msg.post, msg.websocket_id).ok();
  }
}

impl Handler<SendComment> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendComment, _: &mut Context<Self>) {
    self
      .send_comment(&msg.op, &msg.comment, msg.websocket_id)
      .ok();
  }
}

impl Handler<JoinUserRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinUserRoom, _: &mut Context<Self>) {
    self.join_user_room(msg.user_id, msg.id).ok();
  }
}

impl Handler<JoinCommunityRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinCommunityRoom, _: &mut Context<Self>) {
    self.join_community_room(msg.community_id, msg.id).ok();
  }
}

impl Handler<JoinPostRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinPostRoom, _: &mut Context<Self>) {
    self.join_post_room(msg.post_id, msg.id).ok();
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

fn to_json_string<Response>(op: &UserOperation, data: &Response) -> Result<String, LemmyError>
where
  Response: Serialize,
{
  let response = WebsocketResponse {
    op: op.to_string(),
    data,
  };
  Ok(serde_json::to_string(&response)?)
}

impl Handler<CaptchaItem> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: CaptchaItem, _: &mut Context<Self>) {
    self.captchas.push(msg);
  }
}

impl Handler<CheckCaptcha> for ChatServer {
  type Result = bool;

  fn handle(&mut self, msg: CheckCaptcha, _: &mut Context<Self>) -> Self::Result {
    // Remove all the ones that are past the expire time
    self.captchas.retain(|x| x.expires.gt(&naive_now()));

    let check = self
      .captchas
      .iter()
      .any(|r| r.uuid == msg.uuid && r.answer == msg.answer);

    // Remove this uuid so it can't be re-checked (Checks only work once)
    self.captchas.retain(|x| x.uuid != msg.uuid);

    check
  }
}
