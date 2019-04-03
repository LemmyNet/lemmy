//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use bcrypt::{verify};
use std::str::FromStr;

use {Crud, Joinable, Likeable, Followable, establish_connection, naive_now};
use actions::community::*;
use actions::user::*;
use actions::post::*;
use actions::comment::*;
use actions::post_view::*;

#[derive(EnumString,ToString,Debug)]
pub enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, GetPost, GetCommunity, CreateComment, EditComment, CreateCommentLike, GetPosts, CreatePostLike
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
  op: String,
  error: String
}

/// Chat server sends this messages to session
#[derive(Message)]
pub struct WSMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
  pub addr: Recipient<WSMessage>,
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
  pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
pub struct ClientMessage {
  /// Id of the client session
  pub id: usize,
  /// Peer message
  pub msg: String,
  /// Room name
  pub room: String,
}

#[derive(Serialize, Deserialize)]
pub struct StandardMessage {
  /// Id of the client session
  pub id: usize,
  /// Peer message
  pub msg: String,
}

impl actix::Message for StandardMessage {
  type Result = String;
}

#[derive(Serialize, Deserialize)]
pub struct Login {
  pub username_or_email: String,
  pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct Register {
  username: String,
  email: Option<String>,
  password: String,
  password_verify: String
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
  op: String,
  jwt: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommunity {
  name: String,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommunityResponse {
  op: String,
  community: Community
}

#[derive(Serialize, Deserialize)]
pub struct ListCommunities;

#[derive(Serialize, Deserialize)]
pub struct ListCommunitiesResponse {
  op: String,
  communities: Vec<Community>
}

#[derive(Serialize, Deserialize)]
pub struct CreatePost {
  name: String,
  url: Option<String>,
  body: Option<String>,
  community_id: i32,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostResponse {
  op: String,
  post: Post
}


#[derive(Serialize, Deserialize)]
pub struct GetPost {
  id: i32,
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct GetPostResponse {
  op: String,
  post: PostView,
  comments: Vec<CommentView>
}

#[derive(Serialize, Deserialize)]
pub struct GetPosts {
  type_: String,
  sort: String,
  limit: i64,
  community_id: Option<i32>,
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct GetPostsResponse {
  op: String,
  posts: Vec<PostView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunity {
  id: i32
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  op: String,
  community: Community
}

#[derive(Serialize, Deserialize)]
pub struct CreateComment {
  content: String,
  parent_id: Option<i32>,
  edit_id: Option<i32>,
  post_id: i32,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentResponse {
  op: String,
  comment: CommentView
}


#[derive(Serialize, Deserialize)]
pub struct EditComment {
  content: String,
  parent_id: Option<i32>,
  edit_id: i32,
  post_id: i32,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct EditCommentResponse {
  op: String,
  comment: CommentView
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLike {
  comment_id: i32,
  post_id: i32,
  score: i16,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLikeResponse {
  op: String,
  comment: CommentView
}


#[derive(Serialize, Deserialize)]
pub struct CreatePostLike {
  post_id: i32,
  score: i16,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostLikeResponse {
  op: String,
  post: PostView
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
  sessions: HashMap<usize, Recipient<WSMessage>>, // A map from generated random ID to session addr
  rooms: HashMap<i32, HashSet<usize>>, // A map from room / post name to set of connectionIDs
  rng: ThreadRng,
}

impl Default for ChatServer {
  fn default() -> ChatServer {
    // default room
    let rooms = HashMap::new();

    ChatServer {
      sessions: HashMap::new(),
      rooms: rooms,
      rng: rand::thread_rng(),
    }
  }
}

impl ChatServer {
  /// Send message to all users in the room
  fn send_room_message(&self, room: i32, message: &str, skip_id: usize) {
    if let Some(sessions) = self.rooms.get(&room) {
      for id in sessions {
        if *id != skip_id {
          if let Some(addr) = self.sessions.get(id) {
            let _ = addr.do_send(WSMessage(message.to_owned()));
          }
        }
      }
    }
  }

  /// Send message only to self
  fn send(&self, message: &str, id: &usize) {
    // println!("{:?}", self.sessions);
    if let Some(addr) = self.sessions.get(id) {
      println!("msg: {}", message); 
      // println!("{:?}", addr.connected());
      let _ = addr.do_send(WSMessage(message.to_owned()));
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

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    println!("Someone joined");

    // notify all users in same room
    // self.send_room_message(&"Main".to_owned(), "Someone joined", 0);

    // register session with random id
    let id = self.rng.gen::<usize>();
    self.sessions.insert(id, msg.addr);

    // auto join session to Main room
    // self.rooms.get_mut(&"Main".to_owned()).unwrap().insert(id);

    // send id back
    id
  }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    println!("Someone disconnected");

    let mut rooms: Vec<i32> = Vec::new();

    // remove address
    if self.sessions.remove(&msg.id).is_some() {
      // remove session from all rooms
      for (id, sessions) in &mut self.rooms {
        if sessions.remove(&msg.id) {
          // rooms.push(*id);
        }
      }
    }
    // send message to other users
    // for room in rooms {
      // self.send_room_message(room, "Someone disconnected", 0);
    // }
  }
}

/// Handler for Message message.
// impl Handler<ClientMessage> for ChatServer {
//   type Result = ();

//   fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
//     self.send_room_message(&msg.room, msg.msg.as_str(), msg.id);
//   }
// }

/// Handler for Message message.
impl Handler<StandardMessage> for ChatServer {
  type Result = MessageResult<StandardMessage>;

  fn handle(&mut self, msg: StandardMessage, _: &mut Context<Self>) -> Self::Result {

    let json: Value = serde_json::from_str(&msg.msg)
      .expect("Couldn't parse message");

    let data: &Value = &json["data"];
    let op = &json["op"].as_str().unwrap();
    let user_operation: UserOperation = UserOperation::from_str(&op).unwrap();

    let res: String = match user_operation {
      UserOperation::Login => {
        let login: Login = serde_json::from_str(&data.to_string()).unwrap();
        login.perform(self, msg.id)
      },
      UserOperation::Register => {
        let register: Register = serde_json::from_str(&data.to_string()).unwrap();
        register.perform(self, msg.id)
      },
      UserOperation::CreateCommunity => {
        let create_community: CreateCommunity = serde_json::from_str(&data.to_string()).unwrap();
        create_community.perform(self, msg.id)
      },
      UserOperation::ListCommunities => {
        let list_communities: ListCommunities = ListCommunities;
        list_communities.perform(self, msg.id)
      },
      UserOperation::CreatePost => {
        let create_post: CreatePost = serde_json::from_str(&data.to_string()).unwrap();
        create_post.perform(self, msg.id)
      },
      UserOperation::GetPost => {
        let get_post: GetPost = serde_json::from_str(&data.to_string()).unwrap();
        get_post.perform(self, msg.id)
      },
      UserOperation::GetCommunity => {
        let get_community: GetCommunity = serde_json::from_str(&data.to_string()).unwrap();
        get_community.perform(self, msg.id)
      },
      UserOperation::CreateComment => {
        let create_comment: CreateComment = serde_json::from_str(&data.to_string()).unwrap();
        create_comment.perform(self, msg.id)
      },
      UserOperation::EditComment => {
        let edit_comment: EditComment = serde_json::from_str(&data.to_string()).unwrap();
        edit_comment.perform(self, msg.id)
      },
      UserOperation::CreateCommentLike => {
        let create_comment_like: CreateCommentLike = serde_json::from_str(&data.to_string()).unwrap();
        create_comment_like.perform(self, msg.id)
      },
      UserOperation::GetPosts => {
        let get_posts: GetPosts = serde_json::from_str(&data.to_string()).unwrap();
        get_posts.perform(self, msg.id)
      },
      UserOperation::CreatePostLike => {
        let create_post_like: CreatePostLike = serde_json::from_str(&data.to_string()).unwrap();
        create_post_like.perform(self, msg.id)
      },
      _ => {
        let e = ErrorMessage { 
          op: "Unknown".to_string(),
          error: "Unknown User Operation".to_string()
        };
        serde_json::to_string(&e).unwrap()
      }
    };

    MessageResult(res)
  }
}


pub trait Perform {
  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String;
  fn op_type(&self) -> UserOperation;
  fn error(&self, error_msg: &str) -> String {
    serde_json::to_string(
      &ErrorMessage {
        op: self.op_type().to_string(), 
        error: error_msg.to_string()
      }
      )
      .unwrap()
  }
}

impl Perform for Login {
  fn op_type(&self) -> UserOperation {
    UserOperation::Login
  }
  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    // Fetch that username / email
    let user: User_ = match User_::find_by_email_or_username(&conn, &self.username_or_email) {
      Ok(user) => user,
      Err(e) => return self.error("Couldn't find that username or email")
    };

    // Verify the password
    let valid: bool = verify(&self.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return self.error("Password incorrect")
    }

    // Return the jwt
    serde_json::to_string(
      &LoginResponse {
        op: self.op_type().to_string(),
        jwt: user.jwt()
      }
      )
      .unwrap()
  }

}

impl Perform for Register {
  fn op_type(&self) -> UserOperation {
    UserOperation::Register
  }
  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    // Make sure passwords match
    if &self.password != &self.password_verify {
      return self.error("Passwords do not match.");
    }

    // Register the new user
    let user_form = UserForm {
      name: self.username.to_owned(),
      fedi_name: "rrf".into(),
      email: self.email.to_owned(),
      password_encrypted: self.password.to_owned(),
      preferred_username: None,
      updated: None
    };

    // Create the user
    let inserted_user = match User_::create(&conn, &user_form) {
      Ok(user) => user,
      Err(e) => {
        return self.error("User already exists.");
      }
    };

    // Return the jwt
    serde_json::to_string(
      &LoginResponse {
        op: self.op_type().to_string(), 
        jwt: inserted_user.jwt()
      }
      )
      .unwrap()

  }
}

impl Perform for CreateCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateCommunity
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;
    let username = claims.username;
    let iss = claims.iss;

    // When you create a community, make sure the user becomes a moderator and a follower

    let community_form = CommunityForm {
      name: self.name.to_owned(),
      creator_id: user_id,
      updated: None
    };

    let inserted_community = match Community::create(&conn, &community_form) {
      Ok(community) => community,
      Err(e) => {
        return self.error("Community already exists.");
      }
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let inserted_community_moderator = match CommunityModerator::join(&conn, &community_moderator_form) {
      Ok(user) => user,
      Err(e) => {
        return self.error("Community moderator already exists.");
      }
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let inserted_community_follower = match CommunityFollower::follow(&conn, &community_follower_form) {
      Ok(user) => user,
      Err(e) => {
        return self.error("Community follower already exists.");
      }
    };

    serde_json::to_string(
      &CreateCommunityResponse {
        op: self.op_type().to_string(), 
        community: inserted_community
      }
      )
      .unwrap()
  }
}

impl Perform for ListCommunities {
  fn op_type(&self) -> UserOperation {
    UserOperation::ListCommunities
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let communities: Vec<Community> = Community::list_all(&conn).unwrap();

    // Return the jwt
    serde_json::to_string(
      &ListCommunitiesResponse {
        op: self.op_type().to_string(),
        communities: communities
      }
      )
      .unwrap()
  }
}

impl Perform for CreatePost {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreatePost
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;
    let username = claims.username;
    let iss = claims.iss;

    let post_form = PostForm {
      name: self.name.to_owned(),
      url: self.url.to_owned(),
      body: self.body.to_owned(),
      community_id: self.community_id,
      creator_id: user_id,
      updated: None
    };

    let inserted_post = match Post::create(&conn, &post_form) {
      Ok(post) => post,
      Err(e) => {
        return self.error("Couldn't create Post");
      }
    };

    // They like their own post by default
    let like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: user_id,
      score: 1
    };

    // Only add the like if the score isnt 0
    let inserted_like = match PostLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(e) => {
        return self.error("Couldn't like post.");
      }
    };

    serde_json::to_string(
      &CreatePostResponse {
        op: self.op_type().to_string(), 
        post: inserted_post
      }
      )
      .unwrap()
  }
}


impl Perform for GetPost {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetPost
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    println!("{:?}", self.auth);

    let user_id: Option<i32> = match &self.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            let username = claims.claims.username;
            let iss = claims.claims.iss;
            Some(user_id)
          }
          Err(e) => None
        }
      }
      None => None
    };

    let post_view = match PostView::get(&conn, self.id, user_id) {
      Ok(post) => post,
      Err(e) => {
        return self.error("Couldn't find Post");
      }
    };

    // remove session from all rooms
    for (n, sessions) in &mut chat.rooms {
      sessions.remove(&addr);
    }

    if chat.rooms.get_mut(&self.id).is_none() {
      chat.rooms.insert(self.id, HashSet::new());
    }

    chat.rooms.get_mut(&self.id).unwrap().insert(addr);

    let comments = CommentView::from_post(&conn, self.id, user_id);

    // println!("{:?}", chat.rooms.keys());
    // println!("{:?}", chat.rooms.get(&5i32).unwrap());

    // Return the jwt
    serde_json::to_string(
      &GetPostResponse {
        op: self.op_type().to_string(),
        post: post_view,
        comments: comments
      }
      )
      .unwrap()
  }
}

impl Perform for GetCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetCommunity
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let community = match Community::read(&conn, self.id) {
      Ok(community) => community,
      Err(e) => {
        return self.error("Couldn't find Community");
      }
    };

    // Return the jwt
    serde_json::to_string(
      &GetCommunityResponse {
        op: self.op_type().to_string(),
        community: community
      }
      )
      .unwrap()
  }
}

impl Perform for CreateComment {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateComment
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;
    let username = claims.username;
    let iss = claims.iss;
    let fedi_user_id = format!("{}/{}", iss, username);

    let comment_form = CommentForm {
      content: self.content.to_owned(),
      parent_id: self.parent_id.to_owned(),
      post_id: self.post_id,
      creator_id: user_id,
      updated: None
    };

    let inserted_comment = match Comment::create(&conn, &comment_form) {
      Ok(comment) => comment,
      Err(e) => {
        return self.error("Couldn't create Comment");
      }
    };

    // You like your own comment by default
    let like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: self.post_id,
      user_id: user_id,
      score: 1
    };

    let inserted_like = match CommentLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(e) => {
        return self.error("Couldn't like comment.");
      }
    };

    let likes: Vec<CommentLike> = vec![inserted_like];

    let comment_view = CommentView::from_comment(&inserted_comment, &likes, Some(user_id));

    let mut comment_sent = comment_view.clone();
    comment_sent.my_vote = None;

    let comment_out = serde_json::to_string(
      &CreateCommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )
      .unwrap();

    let comment_sent_out = serde_json::to_string(
      &CreateCommentLikeResponse {
        op: self.op_type().to_string(), 
        comment: comment_sent
      }
      )
      .unwrap();
    
    chat.send_room_message(self.post_id, &comment_sent_out, addr);

    comment_out
  }
}

impl Perform for EditComment {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditComment
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;
    let username = claims.username;
    let iss = claims.iss;
    let fedi_user_id = format!("{}/{}", iss, username);

    let comment_form = CommentForm {
      content: self.content.to_owned(),
      parent_id: self.parent_id,
      post_id: self.post_id,
      creator_id: user_id,
      updated: Some(naive_now())
    };

    let updated_comment = match Comment::update(&conn, self.edit_id, &comment_form) {
      Ok(comment) => comment,
      Err(e) => {
        return self.error("Couldn't update Comment");
      }
    };

    let likes = match CommentLike::read(&conn, self.edit_id) {
      Ok(likes) => likes,
      Err(e) => {
        return self.error("Couldn't get likes");
      }
    };

    let comment_view = CommentView::from_comment(&updated_comment, &likes, Some(user_id));

    let mut comment_sent = comment_view.clone();
    comment_sent.my_vote = None;

    let comment_out = serde_json::to_string(
      &CreateCommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )
      .unwrap();

    let comment_sent_out = serde_json::to_string(
      &CreateCommentLikeResponse {
        op: self.op_type().to_string(), 
        comment: comment_sent
      }
      )
      .unwrap();
    
    chat.send_room_message(self.post_id, &comment_sent_out, addr);

    comment_out
  }
}

impl Perform for CreateCommentLike {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateCommentLike
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;
    let username = claims.username;
    let iss = claims.iss;
    let fedi_user_id = format!("{}/{}", iss, username);

    let like_form = CommentLikeForm {
      comment_id: self.comment_id,
      post_id: self.post_id,
      user_id: user_id,
      score: self.score
    };

    // Remove any likes first
    CommentLike::remove(&conn, &like_form).unwrap();

    // Only add the like if the score isnt 0
    if &like_form.score != &0 {
      let inserted_like = match CommentLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(e) => {
          return self.error("Couldn't like comment.");
        }
      };
    }

    // Have to refetch the comment to get the current state
    // thread::sleep(time::Duration::from_secs(1));
    let liked_comment = CommentView::read(&conn, self.comment_id, Some(user_id));

    let mut liked_comment_sent = liked_comment.clone();
    liked_comment_sent.my_vote = None;

    let like_out = serde_json::to_string(
      &CreateCommentLikeResponse {
        op: self.op_type().to_string(), 
        comment: liked_comment
      }
      )
      .unwrap();

    let like_sent_out = serde_json::to_string(
      &CreateCommentLikeResponse {
        op: self.op_type().to_string(), 
        comment: liked_comment_sent
      }
      )
      .unwrap();

      chat.send_room_message(self.post_id, &like_sent_out, addr);

      like_out
  }
}


impl Perform for GetPosts {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetPosts
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    println!("{:?}", self.auth);

    let user_id: Option<i32> = match &self.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            Some(user_id)
          }
          Err(e) => None
        }
      }
      None => None
    };

    let type_ = ListingType::from_str(&self.type_).expect("listing type");
    let sort = ListingSortType::from_str(&self.sort).expect("listing sort");

    let posts = match PostView::list(&conn, type_, sort, self.community_id, user_id, self.limit) {
      Ok(posts) => posts,
      Err(e) => {
        eprintln!("{}", e);
        return self.error("Couldn't get posts");
      }
    };

    // Return the jwt
    serde_json::to_string(
      &GetPostsResponse {
        op: self.op_type().to_string(),
        posts: posts
      }
      )
      .unwrap()
  }
}


impl Perform for CreatePostLike {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreatePostLike
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    let like_form = PostLikeForm {
      post_id: self.post_id,
      user_id: user_id,
      score: self.score
    };

    // Remove any likes first
    PostLike::remove(&conn, &like_form).unwrap();

    // Only add the like if the score isnt 0
    if &like_form.score != &0 {
      let inserted_like = match PostLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(e) => {
          return self.error("Couldn't like post.");
        }
      };
    }

    let post_view = match PostView::get(&conn, self.post_id, Some(user_id)) {
      Ok(post) => post,
      Err(e) => {
        return self.error("Couldn't find Post");
      }
    };

    // just output the score

    let like_out = serde_json::to_string(
      &CreatePostLikeResponse {
        op: self.op_type().to_string(), 
        post: post_view
      }
      )
      .unwrap();

      like_out
  }
}

// impl Handler<Login> for ChatServer {

//   type Result = MessageResult<Login>;
//   fn handle(&mut self, msg: Login, _: &mut Context<Self>) -> Self::Result {

//     let conn = establish_connection();

//     // Fetch that username / email
//     let user: User_ = match User_::find_by_email_or_username(&conn, &msg.username_or_email) {
//       Ok(user) => user,
//       Err(e) => return MessageResult(
//         Err(
//           ErrorMessage {
//             op: UserOperation::Login.to_string(), 
//             error: "Couldn't find that username or email".to_string()
//           }
//           )
//         )
//     };

//     // Verify the password
//     let valid: bool = verify(&msg.password, &user.password_encrypted).unwrap_or(false);
//     if !valid {
//       return MessageResult(
//         Err(
//           ErrorMessage {
//             op: UserOperation::Login.to_string(), 
//             error: "Password incorrect".to_string()
//           }
//           )
//         )
//     }

//     // Return the jwt
//     MessageResult(
//       Ok(
//         LoginResponse {
//           op: UserOperation::Login.to_string(), 
//           jwt: user.jwt()
//         }
//         )
//       )
//   }
// }

// impl Handler<Register> for ChatServer {

//   type Result = MessageResult<Register>;
//   fn handle(&mut self, msg: Register, _: &mut Context<Self>) -> Self::Result {

//     let conn = establish_connection();

//     // Make sure passwords match
//     if msg.password != msg.password_verify {
//       return MessageResult(
//         Err(
//           ErrorMessage {
//             op: UserOperation::Register.to_string(), 
//             error: "Passwords do not match.".to_string()
//           }
//           )
//         );
//     }

//     // Register the new user
//     let user_form = UserForm {
//       name: msg.username,
//       email: msg.email,
//       password_encrypted: msg.password,
//       preferred_username: None,
//       updated: None
//     };

//     // Create the user
//     let inserted_user = match User_::create(&conn, &user_form) {
//       Ok(user) => user,
//       Err(e) => return MessageResult(
//         Err(
//           ErrorMessage {
//             op: UserOperation::Register.to_string(), 
//             error: "User already exists.".to_string() // overwrite the diesel error
//           }
//           )
//         )
//     };

//     // Return the jwt
//     MessageResult(
//       Ok(
//         LoginResponse {
//           op: UserOperation::Register.to_string(), 
//           jwt: inserted_user.jwt()
//         }
//         )
//       )

//   }
// }


// impl Handler<CreateCommunity> for ChatServer {

//   type Result = MessageResult<CreateCommunity>;

//   fn handle(&mut self, msg: CreateCommunity, _: &mut Context<Self>) -> Self::Result {
//     let conn = establish_connection();

//     let user_id = Claims::decode(&msg.auth).id;

//     let community_form = CommunityForm {
//       name: msg.name,
//       updated: None
//     };

//     let community = match Community::create(&conn, &community_form) {
//       Ok(community) => community,
//       Err(e) => return MessageResult(
//         Err(
//           ErrorMessage {
//             op: UserOperation::CreateCommunity.to_string(), 
//             error: "Community already exists.".to_string() // overwrite the diesel error
//           }
//           )
//         )
//     };
    
//     MessageResult(
//       Ok(
//         CreateCommunityResponse {
//           op: UserOperation::CreateCommunity.to_string(), 
//           community: community
//         }
//         )
//       )
//   }
// }
//
//
//
// /// Handler for `ListRooms` message.
// impl Handler<ListRooms> for ChatServer {
//   type Result = MessageResult<ListRooms>;

//   fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
//     let mut rooms = Vec::new();

//     for key in self.rooms.keys() {
//       rooms.push(key.to_owned())
//     }

//     MessageResult(rooms)
//   }
// }

// /// Join room, send disconnect message to old room
// /// send join message to new room
// impl Handler<Join> for ChatServer {
//   type Result = ();

//   fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
//     let Join { id, name } = msg;
//     let mut rooms = Vec::new();

//     // remove session from all rooms
//     for (n, sessions) in &mut self.rooms {
//       if sessions.remove(&id) {
//         rooms.push(n.to_owned());
//       }
//     }
//     // send message to other users
//     for room in rooms {
//       self.send_room_message(&room, "Someone disconnected", 0);
//     }

//     if self.rooms.get_mut(&name).is_none() {
//       self.rooms.insert(name.clone(), HashSet::new());
//     }
//     self.send_room_message(&name, "Someone connected", id);
//     self.rooms.get_mut(&name).unwrap().insert(id);
//   }

// }
