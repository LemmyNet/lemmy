//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;
use failure::Error;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::SystemTime;

use crate::api::comment::*;
use crate::api::community::*;
use crate::api::post::*;
use crate::api::site::*;
use crate::api::user::*;
use crate::api::*;
use crate::websocket::UserOperation;
use crate::Settings;

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
  pub ip: String,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub id: usize,
  pub ip: String,
}

// TODO this is unused rn
/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
  /// Id of the client session
  pub id: usize,
  /// Peer message
  pub msg: String,
  /// Room name
  pub room: String,
}

#[derive(Serialize, Deserialize, Message)]
#[rtype(String)]
pub struct StandardMessage {
  /// Id of the client session
  pub id: usize,
  /// Peer message
  pub msg: String,
}

#[derive(Debug)]
pub struct RateLimitBucket {
  last_checked: SystemTime,
  allowance: f64,
}

pub struct SessionInfo {
  pub addr: Recipient<WSMessage>,
  pub ip: String,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
  sessions: HashMap<usize, SessionInfo>, // A map from generated random ID to session addr
  rate_limits: HashMap<String, RateLimitBucket>,
  rooms: HashMap<i32, HashSet<usize>>, // A map from room / post name to set of connectionIDs
  rng: ThreadRng,
  db: Pool<ConnectionManager<PgConnection>>,
}

impl ChatServer {
  pub fn startup(db: Pool<ConnectionManager<PgConnection>>) -> ChatServer {
    // default room
    let rooms = HashMap::new();

    ChatServer {
      sessions: HashMap::new(),
      rate_limits: HashMap::new(),
      rooms,
      rng: rand::thread_rng(),
      db,
    }
  }

  /// Send message to all users in the room
  fn send_room_message(&self, room: i32, message: &str, skip_id: usize) {
    if let Some(sessions) = self.rooms.get(&room) {
      for id in sessions {
        if *id != skip_id {
          if let Some(info) = self.sessions.get(id) {
            let _ = info.addr.do_send(WSMessage(message.to_owned()));
          }
        }
      }
    }
  }

  fn join_room(&mut self, room_id: i32, id: usize) {
    // remove session from all rooms
    for sessions in self.rooms.values_mut() {
      sessions.remove(&id);
    }

    // If the room doesn't exist yet
    if self.rooms.get_mut(&room_id).is_none() {
      self.rooms.insert(room_id, HashSet::new());
    }

    self.rooms.get_mut(&room_id).unwrap().insert(id);
  }

  fn send_community_message(
    &self,
    community_id: i32,
    message: &str,
    skip_id: usize,
  ) -> Result<(), Error> {
    use crate::db::post_view::*;
    use crate::db::*;

    let conn = self.db.get()?;

    let posts = PostQueryBuilder::create(&conn)
      .listing_type(ListingType::Community)
      .sort(&SortType::New)
      .for_community_id(community_id)
      .limit(9999)
      .list()?;

    for post in posts {
      self.send_room_message(post.id, message, skip_id);
    }

    Ok(())
  }

  fn check_rate_limit_register(&mut self, id: usize) -> Result<(), Error> {
    self.check_rate_limit_full(
      id,
      Settings::get().rate_limit.register,
      Settings::get().rate_limit.register_per_second,
    )
  }

  fn check_rate_limit_post(&mut self, id: usize) -> Result<(), Error> {
    self.check_rate_limit_full(
      id,
      Settings::get().rate_limit.post,
      Settings::get().rate_limit.post_per_second,
    )
  }

  fn check_rate_limit_message(&mut self, id: usize) -> Result<(), Error> {
    self.check_rate_limit_full(
      id,
      Settings::get().rate_limit.message,
      Settings::get().rate_limit.message_per_second,
    )
  }

  #[allow(clippy::float_cmp)]
  fn check_rate_limit_full(&mut self, id: usize, rate: i32, per: i32) -> Result<(), Error> {
    if let Some(info) = self.sessions.get(&id) {
      if let Some(rate_limit) = self.rate_limits.get_mut(&info.ip) {
        // The initial value
        if rate_limit.allowance == -2f64 {
          rate_limit.allowance = rate as f64;
        };

        let current = SystemTime::now();
        let time_passed = current.duration_since(rate_limit.last_checked)?.as_secs() as f64;
        rate_limit.last_checked = current;
        rate_limit.allowance += time_passed * (rate as f64 / per as f64);
        if rate_limit.allowance > rate as f64 {
          rate_limit.allowance = rate as f64;
        }

        if rate_limit.allowance < 1.0 {
          println!(
            "Rate limited IP: {}, time_passed: {}, allowance: {}",
            &info.ip, time_passed, rate_limit.allowance
          );
          Err(
            APIError {
              message: format!("Too many requests. {} per {} seconds", rate, per),
            }
            .into(),
          )
        } else {
          rate_limit.allowance -= 1.0;
          Ok(())
        }
      } else {
        Ok(())
      }
    } else {
      Ok(())
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
    // notify all users in same room
    // self.send_room_message(&"Main".to_owned(), "Someone joined", 0);

    // register session with random id
    let id = self.rng.gen::<usize>();
    println!("{} joined", &msg.ip);

    self.sessions.insert(
      id,
      SessionInfo {
        addr: msg.addr,
        ip: msg.ip.to_owned(),
      },
    );

    if self.rate_limits.get(&msg.ip).is_none() {
      self.rate_limits.insert(
        msg.ip,
        RateLimitBucket {
          last_checked: SystemTime::now(),
          allowance: -2f64,
        },
      );
    }

    id
  }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    // let mut rooms: Vec<i32> = Vec::new();

    // remove address
    if self.sessions.remove(&msg.id).is_some() {
      // remove session from all rooms
      for sessions in self.rooms.values_mut() {
        if sessions.remove(&msg.id) {
          // rooms.push(*id);
        }
      }
    }
  }
}

/// Handler for Message message.
impl Handler<StandardMessage> for ChatServer {
  type Result = MessageResult<StandardMessage>;

  fn handle(&mut self, msg: StandardMessage, _: &mut Context<Self>) -> Self::Result {
    let msg_out = match parse_json_message(self, msg) {
      Ok(m) => m,
      Err(e) => e.to_string(),
    };

    println!("Message Sent: {}", msg_out);
    MessageResult(msg_out)
  }
}

#[derive(Serialize)]
struct WebsocketResponse<T> {
  op: String,
  data: T,
}

fn to_json_string<T>(op: &UserOperation, data: T) -> Result<String, Error>
where
  T: Serialize,
{
  let response = WebsocketResponse {
    op: op.to_string(),
    data,
  };
  Ok(serde_json::to_string(&response)?)
}

fn do_user_operation<'a, Data, Response>(
  op: UserOperation,
  data: &str,
  conn: &PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<String, Error>
where
  for<'de> Data: Deserialize<'de> + 'a,
  Response: Serialize,
  Oper<Data>: Perform<Response>,
{
  let parsed_data: Data = serde_json::from_str(data)?;
  let res = Oper::new(parsed_data).perform(&conn)?;
  to_json_string(&op, &res)
}

fn parse_json_message(chat: &mut ChatServer, msg: StandardMessage) -> Result<String, Error> {
  let json: Value = serde_json::from_str(&msg.msg)?;
  let data = &json["data"].to_string();
  let op = &json["op"].as_str().ok_or(APIError {
    message: "Unknown op type".to_string(),
  })?;

  let conn = chat.db.get()?;

  let user_operation: UserOperation = UserOperation::from_str(&op)?;

  // TODO: none of the chat messages are going to work if stuff is submitted via http api,
  //       need to move that handling elsewhere
  match user_operation {
    UserOperation::Login => do_user_operation::<Login, LoginResponse>(user_operation, data, &conn),
    UserOperation::Register => {
      chat.check_rate_limit_register(msg.id)?;
      do_user_operation::<Register, LoginResponse>(user_operation, data, &conn)
    }
    UserOperation::GetUserDetails => {
      do_user_operation::<GetUserDetails, GetUserDetailsResponse>(user_operation, data, &conn)
    }
    UserOperation::SaveUserSettings => {
      do_user_operation::<SaveUserSettings, LoginResponse>(user_operation, data, &conn)
    }
    UserOperation::AddAdmin => {
      do_user_operation::<AddAdmin, AddAdminResponse>(user_operation, data, &conn)
    }
    UserOperation::BanUser => {
      do_user_operation::<BanUser, BanUserResponse>(user_operation, data, &conn)
    }
    UserOperation::GetReplies => {
      do_user_operation::<GetReplies, GetRepliesResponse>(user_operation, data, &conn)
    }
    UserOperation::GetUserMentions => {
      do_user_operation::<GetUserMentions, GetUserMentionsResponse>(user_operation, data, &conn)
    }
    UserOperation::EditUserMention => {
      do_user_operation::<EditUserMention, UserMentionResponse>(user_operation, data, &conn)
    }
    UserOperation::MarkAllAsRead => {
      do_user_operation::<MarkAllAsRead, GetRepliesResponse>(user_operation, data, &conn)
    }
    UserOperation::GetCommunity => {
      do_user_operation::<GetCommunity, GetCommunityResponse>(user_operation, data, &conn)
    }
    UserOperation::ListCommunities => {
      do_user_operation::<ListCommunities, ListCommunitiesResponse>(user_operation, data, &conn)
    }
    UserOperation::CreateCommunity => {
      chat.check_rate_limit_register(msg.id)?;
      do_user_operation::<CreateCommunity, CommunityResponse>(user_operation, data, &conn)
    }
    UserOperation::EditCommunity => {
      let edit_community: EditCommunity = serde_json::from_str(data)?;
      let res = Oper::new(edit_community).perform(&conn)?;
      let mut community_sent: CommunityResponse = res.clone();
      community_sent.community.user_id = None;
      community_sent.community.subscribed = None;
      let community_sent_str = to_json_string(&user_operation, &community_sent)?;
      chat.send_community_message(community_sent.community.id, &community_sent_str, msg.id)?;
      to_json_string(&user_operation, &res)
    }
    UserOperation::FollowCommunity => {
      do_user_operation::<FollowCommunity, CommunityResponse>(user_operation, data, &conn)
    }
    UserOperation::GetFollowedCommunities => do_user_operation::<
      GetFollowedCommunities,
      GetFollowedCommunitiesResponse,
    >(user_operation, data, &conn),
    UserOperation::BanFromCommunity => {
      let ban_from_community: BanFromCommunity = serde_json::from_str(data)?;
      let community_id = ban_from_community.community_id;
      let res = Oper::new(ban_from_community).perform(&conn)?;
      let res_str = to_json_string(&user_operation, &res)?;
      chat.send_community_message(community_id, &res_str, msg.id)?;
      Ok(res_str)
    }
    UserOperation::AddModToCommunity => {
      let mod_add_to_community: AddModToCommunity = serde_json::from_str(data)?;
      let community_id = mod_add_to_community.community_id;
      let res = Oper::new(mod_add_to_community).perform(&conn)?;
      let res_str = to_json_string(&user_operation, &res)?;
      chat.send_community_message(community_id, &res_str, msg.id)?;
      Ok(res_str)
    }
    UserOperation::ListCategories => {
      do_user_operation::<ListCategories, ListCategoriesResponse>(user_operation, data, &conn)
    }
    UserOperation::CreatePost => {
      chat.check_rate_limit_post(msg.id)?;
      do_user_operation::<CreatePost, PostResponse>(user_operation, data, &conn)
    }
    UserOperation::GetPost => {
      let get_post: GetPost = serde_json::from_str(data)?;
      chat.join_room(get_post.id, msg.id);
      let res = Oper::new(get_post).perform(&conn)?;
      to_json_string(&user_operation, &res)
    }
    UserOperation::GetPosts => {
      do_user_operation::<GetPosts, GetPostsResponse>(user_operation, data, &conn)
    }
    UserOperation::CreatePostLike => {
      chat.check_rate_limit_message(msg.id)?;
      do_user_operation::<CreatePostLike, CreatePostLikeResponse>(user_operation, data, &conn)
    }
    UserOperation::EditPost => {
      let edit_post: EditPost = serde_json::from_str(data)?;
      let res = Oper::new(edit_post).perform(&conn)?;
      let mut post_sent = res.clone();
      post_sent.post.my_vote = None;
      let post_sent_str = to_json_string(&user_operation, &post_sent)?;
      chat.send_room_message(post_sent.post.id, &post_sent_str, msg.id);
      to_json_string(&user_operation, &res)
    }
    UserOperation::SavePost => {
      do_user_operation::<SavePost, PostResponse>(user_operation, data, &conn)
    }
    UserOperation::CreateComment => {
      chat.check_rate_limit_message(msg.id)?;
      let create_comment: CreateComment = serde_json::from_str(data)?;
      let post_id = create_comment.post_id;
      let res = Oper::new(create_comment).perform(&conn)?;
      let mut comment_sent = res.clone();
      comment_sent.comment.my_vote = None;
      comment_sent.comment.user_id = None;
      let comment_sent_str = to_json_string(&user_operation, &comment_sent)?;
      chat.send_room_message(post_id, &comment_sent_str, msg.id);
      to_json_string(&user_operation, &res)
    }
    UserOperation::EditComment => {
      let edit_comment: EditComment = serde_json::from_str(data)?;
      let post_id = edit_comment.post_id;
      let res = Oper::new(edit_comment).perform(&conn)?;
      let mut comment_sent = res.clone();
      comment_sent.comment.my_vote = None;
      comment_sent.comment.user_id = None;
      let comment_sent_str = to_json_string(&user_operation, &comment_sent)?;
      chat.send_room_message(post_id, &comment_sent_str, msg.id);
      to_json_string(&user_operation, &res)
    }
    UserOperation::SaveComment => {
      do_user_operation::<SaveComment, CommentResponse>(user_operation, data, &conn)
    }
    UserOperation::CreateCommentLike => {
      chat.check_rate_limit_message(msg.id)?;
      let create_comment_like: CreateCommentLike = serde_json::from_str(data)?;
      let post_id = create_comment_like.post_id;
      let res = Oper::new(create_comment_like).perform(&conn)?;
      let mut comment_sent = res.clone();
      comment_sent.comment.my_vote = None;
      comment_sent.comment.user_id = None;
      let comment_sent_str = to_json_string(&user_operation, &comment_sent)?;
      chat.send_room_message(post_id, &comment_sent_str, msg.id);
      to_json_string(&user_operation, &res)
    }
    UserOperation::GetModlog => {
      do_user_operation::<GetModlog, GetModlogResponse>(user_operation, data, &conn)
    }
    UserOperation::CreateSite => {
      do_user_operation::<CreateSite, SiteResponse>(user_operation, data, &conn)
    }
    UserOperation::EditSite => {
      do_user_operation::<EditSite, SiteResponse>(user_operation, data, &conn)
    }
    UserOperation::GetSite => {
      let online: usize = chat.sessions.len();
      let get_site: GetSite = serde_json::from_str(data)?;
      let mut res = Oper::new(get_site).perform(&conn)?;
      res.online = online;
      to_json_string(&user_operation, &res)
    }
    UserOperation::Search => {
      do_user_operation::<Search, SearchResponse>(user_operation, data, &conn)
    }
    UserOperation::TransferCommunity => {
      do_user_operation::<TransferCommunity, GetCommunityResponse>(user_operation, data, &conn)
    }
    UserOperation::TransferSite => {
      do_user_operation::<TransferSite, GetSiteResponse>(user_operation, data, &conn)
    }
    UserOperation::DeleteAccount => {
      do_user_operation::<DeleteAccount, LoginResponse>(user_operation, data, &conn)
    }
    UserOperation::PasswordReset => {
      do_user_operation::<PasswordReset, PasswordResetResponse>(user_operation, data, &conn)
    }
    UserOperation::PasswordChange => {
      do_user_operation::<PasswordChange, LoginResponse>(user_operation, data, &conn)
    }
    UserOperation::CreatePrivateMessage => {
      chat.check_rate_limit_message(msg.id)?;
      do_user_operation::<CreatePrivateMessage, PrivateMessageResponse>(user_operation, data, &conn)
    }
    UserOperation::EditPrivateMessage => {
      do_user_operation::<EditPrivateMessage, PrivateMessageResponse>(user_operation, data, &conn)
    }
    UserOperation::GetPrivateMessages => {
      do_user_operation::<GetPrivateMessages, PrivateMessagesResponse>(user_operation, data, &conn)
    }
  }
}
