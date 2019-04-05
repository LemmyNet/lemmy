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
use actions::comment_view::*;
use actions::category::*;
use actions::community_view::*;

#[derive(EnumString,ToString,Debug)]
pub enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, EditCommunity, FollowCommunity, GetFollowedCommunities
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
  title: String,
  description: Option<String>,
  category_id: i32 ,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CommunityResponse {
  op: String,
  community: CommunityView
}

#[derive(Serialize, Deserialize)]
pub struct ListCommunities {
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct ListCommunitiesResponse {
  op: String,
  communities: Vec<CommunityView>
}

#[derive(Serialize, Deserialize)]
pub struct ListCategories;

#[derive(Serialize, Deserialize)]
pub struct ListCategoriesResponse {
  op: String,
  categories: Vec<Category>
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
pub struct PostResponse {
  op: String,
  post: PostView
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
  comments: Vec<CommentView>,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>
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
  id: i32,
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  op: String,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>
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
pub struct EditComment {
  content: String,
  parent_id: Option<i32>,
  edit_id: i32,
  post_id: i32,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct CommentResponse {
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


#[derive(Serialize, Deserialize)]
pub struct EditPost {
  edit_id: i32,
  community_id: i32,
  name: String,
  url: Option<String>,
  body: Option<String>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct EditCommunity {
  edit_id: i32,
  name: String,
  title: String,
  description: Option<String>,
  category_id: i32,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct FollowCommunity {
  community_id: i32,
  follow: bool,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunities {
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunitiesResponse {
  op: String,
  communities: Vec<CommunityFollowerView>
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

  // /// Send message only to self
  // fn send(&self, message: &str, id: &usize) {
  //   // println!("{:?}", self.sessions);
  //   if let Some(addr) = self.sessions.get(id) {
  //     println!("msg: {}", message); 
  //     // println!("{:?}", addr.connected());
  //     let _ = addr.do_send(WSMessage(message.to_owned()));
  //   }
  // }
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

    // let mut rooms: Vec<i32> = Vec::new();

    // remove address
    if self.sessions.remove(&msg.id).is_some() {
      // remove session from all rooms
      for (_id, sessions) in &mut self.rooms {
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
        let list_communities: ListCommunities = serde_json::from_str(&data.to_string()).unwrap();
        list_communities.perform(self, msg.id)
      },
      UserOperation::ListCategories => {
        let list_categories: ListCategories = ListCategories;
        list_categories.perform(self, msg.id)
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
      UserOperation::EditPost => {
        let edit_post: EditPost = serde_json::from_str(&data.to_string()).unwrap();
        edit_post.perform(self, msg.id)
      },
      UserOperation::EditCommunity => {
        let edit_community: EditCommunity = serde_json::from_str(&data.to_string()).unwrap();
        edit_community.perform(self, msg.id)
      },
      UserOperation::FollowCommunity => {
        let follow_community: FollowCommunity = serde_json::from_str(&data.to_string()).unwrap();
        follow_community.perform(self, msg.id)
      },
      UserOperation::GetFollowedCommunities => {
        let followed_communities: GetFollowedCommunities = serde_json::from_str(&data.to_string()).unwrap();
        followed_communities.perform(self, msg.id)
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
  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String;
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
  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    // Fetch that username / email
    let user: User_ = match User_::find_by_email_or_username(&conn, &self.username_or_email) {
      Ok(user) => user,
      Err(_e) => return self.error("Couldn't find that username or email")
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
  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

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
      Err(_e) => {
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

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    // When you create a community, make sure the user becomes a moderator and a follower

    let community_form = CommunityForm {
      name: self.name.to_owned(),
      title: self.title.to_owned(),
      description: self.description.to_owned(),
      category_id: self.category_id,
      creator_id: user_id,
      updated: None
    };

    let inserted_community = match Community::create(&conn, &community_form) {
      Ok(community) => community,
      Err(_e) => {
        return self.error("Community already exists.");
      }
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let _inserted_community_moderator = match CommunityModerator::join(&conn, &community_moderator_form) {
      Ok(user) => user,
      Err(_e) => {
        return self.error("Community moderator already exists.");
      }
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let _inserted_community_follower = match CommunityFollower::follow(&conn, &community_follower_form) {
      Ok(user) => user,
      Err(_e) => {
        return self.error("Community follower already exists.");
      }
    };

    let community_view = CommunityView::read(&conn, inserted_community.id, Some(user_id)).unwrap();

    serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view
      }
      )
      .unwrap()
  }
}

impl Perform for ListCommunities {
  fn op_type(&self) -> UserOperation {
    UserOperation::ListCommunities
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let user_id: Option<i32> = match &self.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            Some(user_id)
          }
          Err(_e) => None
        }
      }
      None => None
    };

    let communities: Vec<CommunityView> = CommunityView::list_all(&conn, user_id).unwrap();

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

impl Perform for ListCategories {
  fn op_type(&self) -> UserOperation {
    UserOperation::ListCategories
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let categories: Vec<Category> = Category::list_all(&conn).unwrap();

    // Return the jwt
    serde_json::to_string(
      &ListCategoriesResponse {
        op: self.op_type().to_string(),
        categories: categories
      }
      )
      .unwrap()
  }
}

impl Perform for CreatePost {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreatePost
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

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
      Err(_e) => {
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
    let _inserted_like = match PostLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(_e) => {
        return self.error("Couldn't like post.");
      }
    };

    // Refetch the view
    let post_view = match PostView::read(&conn, inserted_post.id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => {
        return self.error("Couldn't find Post");
      }
    };

    serde_json::to_string(
      &PostResponse {
        op: self.op_type().to_string(), 
        post: post_view
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
            Some(user_id)
          }
          Err(_e) => None
        }
      }
      None => None
    };

    let post_view = match PostView::read(&conn, self.id, user_id) {
      Ok(post) => post,
      Err(_e) => {
        return self.error("Couldn't find Post");
      }
    };

    // remove session from all rooms
    for (_n, sessions) in &mut chat.rooms {
      sessions.remove(&addr);
    }

    if chat.rooms.get_mut(&self.id).is_none() {
      chat.rooms.insert(self.id, HashSet::new());
    }

    chat.rooms.get_mut(&self.id).unwrap().insert(addr);

    let comments = CommentView::list(&conn, self.id, user_id).unwrap();

    let community = CommunityView::read(&conn, post_view.community_id, user_id).unwrap();

    let moderators = CommunityModeratorView::for_community(&conn, post_view.community_id).unwrap();

    // Return the jwt
    serde_json::to_string(
      &GetPostResponse {
        op: self.op_type().to_string(),
        post: post_view,
        comments: comments,
        community: community,
        moderators: moderators
      }
      )
      .unwrap()
  }
}

impl Perform for GetCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetCommunity
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let user_id: Option<i32> = match &self.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            Some(user_id)
          }
          Err(_e) => None
        }
      }
      None => None
    };

    let community_view = match CommunityView::read(&conn, self.id, user_id) {
      Ok(community) => community,
      Err(_e) => {
        return self.error("Couldn't find Community");
      }
    };

    let moderators = match CommunityModeratorView::for_community(&conn, self.id) {
      Ok(moderators) => moderators,
      Err(_e) => {
        return self.error("Couldn't find Community");
      }
    };

    // Return the jwt
    serde_json::to_string(
      &GetCommunityResponse {
        op: self.op_type().to_string(),
        community: community_view,
        moderators: moderators
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
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    let comment_form = CommentForm {
      content: self.content.to_owned(),
      parent_id: self.parent_id.to_owned(),
      post_id: self.post_id,
      creator_id: user_id,
      updated: None
    };

    let inserted_comment = match Comment::create(&conn, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => {
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

    let _inserted_like = match CommentLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(_e) => {
        return self.error("Couldn't like comment.");
      }
    };

    let comment_view = CommentView::read(&conn, inserted_comment.id, Some(user_id)).unwrap();

    let mut comment_sent = comment_view.clone();
    comment_sent.my_vote = None;
    comment_sent.user_id = None;

    let comment_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )
      .unwrap();

    let comment_sent_out = serde_json::to_string(
      &CommentResponse {
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
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    // Verify its the creator
    let orig_comment = Comment::read(&conn, self.edit_id).unwrap();
    if user_id != orig_comment.creator_id {
      return self.error("Incorrect creator.");
    }

    let comment_form = CommentForm {
      content: self.content.to_owned(),
      parent_id: self.parent_id,
      post_id: self.post_id,
      creator_id: user_id,
      updated: Some(naive_now())
    };

    let _updated_comment = match Comment::update(&conn, self.edit_id, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => {
        return self.error("Couldn't update Comment");
      }
    };


    let comment_view = CommentView::read(&conn, self.edit_id, Some(user_id)).unwrap();

    let mut comment_sent = comment_view.clone();
    comment_sent.my_vote = None;
    comment_sent.user_id = None;

    let comment_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )
      .unwrap();

    let comment_sent_out = serde_json::to_string(
      &CommentResponse {
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
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

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
      let _inserted_like = match CommentLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => {
          return self.error("Couldn't like comment.");
        }
      };
    }

    // Have to refetch the comment to get the current state
    let liked_comment = CommentView::read(&conn, self.comment_id, Some(user_id)).unwrap();

    let mut liked_comment_sent = liked_comment.clone();
    liked_comment_sent.my_vote = None;
    liked_comment_sent.user_id = None;

    let like_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: liked_comment
      }
      )
      .unwrap();

    let like_sent_out = serde_json::to_string(
      &CommentResponse {
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

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let user_id: Option<i32> = match &self.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            Some(user_id)
          }
          Err(_e) => None
        }
      }
      None => None
    };

    let type_ = ListingType::from_str(&self.type_).expect("listing type");
    let sort = ListingSortType::from_str(&self.sort).expect("listing sort");

    let posts = match PostView::list(&conn, type_, sort, self.community_id, user_id, self.limit) {
      Ok(posts) => posts,
      Err(_e) => {
        eprintln!("{}", _e);
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

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
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
      let _inserted_like = match PostLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => {
          return self.error("Couldn't like post.");
        }
      };
    }

    let post_view = match PostView::read(&conn, self.post_id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => {
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

impl Perform for EditPost {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditPost
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    // Verify its the creator
    let orig_post = Post::read(&conn, self.edit_id).unwrap();
    if user_id != orig_post.creator_id {
      return self.error("Incorrect creator.");
    }

    let post_form = PostForm {
      name: self.name.to_owned(),
      url: self.url.to_owned(),
      body: self.body.to_owned(),
      creator_id: user_id,
      community_id: self.community_id,
      updated: Some(naive_now())
    };

    let _updated_post = match Post::update(&conn, self.edit_id, &post_form) {
      Ok(post) => post,
      Err(_e) => {
        return self.error("Couldn't update Post");
      }
    };

    let post_view = PostView::read(&conn, self.edit_id, Some(user_id)).unwrap();

    let mut post_sent = post_view.clone();
    post_sent.my_vote = None;

    let post_out = serde_json::to_string(
      &PostResponse {
        op: self.op_type().to_string(), 
        post: post_view
      }
      )
      .unwrap();

    let post_sent_out = serde_json::to_string(
      &PostResponse {
        op: self.op_type().to_string(), 
        post: post_sent
      }
      )
      .unwrap();

    chat.send_room_message(self.edit_id, &post_sent_out, addr);

    post_out
  }
}

impl Perform for EditCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditCommunity
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;


    // Verify its a mod
    let moderator_view = CommunityModeratorView::for_community(&conn, self.edit_id).unwrap();
    let mod_ids: Vec<i32> = moderator_view.into_iter().map(|m| m.user_id).collect();
    if !mod_ids.contains(&user_id) {
      return self.error("Incorrect creator.");
    };

    let community_form = CommunityForm {
      name: self.name.to_owned(),
      title: self.title.to_owned(),
      description: self.description.to_owned(),
      category_id: self.category_id.to_owned(),
      creator_id: user_id,
      updated: Some(naive_now())
    };

    let _updated_community = match Community::update(&conn, self.edit_id, &community_form) {
      Ok(community) => community,
      Err(_e) => {
        return self.error("Couldn't update Community");
      }
    };

    let community_view = CommunityView::read(&conn, self.edit_id, Some(user_id)).unwrap();

    // Do the subscriber stuff here
    // let mut community_sent = post_view.clone();
    // community_sent.my_vote = None;

    let community_out = serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view
      }
      )
      .unwrap();

    // let post_sent_out = serde_json::to_string(
    //   &PostResponse {
    //     op: self.op_type().to_string(), 
    //     post: post_sent
    //   }
    //   )
    //   .unwrap();

    chat.send_room_message(self.edit_id, &community_out, addr);

    community_out
  }
}


impl Perform for FollowCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::FollowCommunity
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    let community_follower_form = CommunityFollowerForm {
      community_id: self.community_id,
      user_id: user_id
    };

    if self.follow {

      match CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => {
          return self.error("Community follower already exists.");
        }
      };
    } else {
      match CommunityFollower::ignore(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => {
          return self.error("Community follower already exists.");
        }
      };
    }

    let community_view = CommunityView::read(&conn, self.community_id, Some(user_id)).unwrap();

    serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view
      }
      )
      .unwrap()
  }
}

impl Perform for GetFollowedCommunities {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetFollowedCommunities
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    let communities: Vec<CommunityFollowerView> = CommunityFollowerView::for_user(&conn, user_id).unwrap();

    // Return the jwt
    serde_json::to_string(
      &GetFollowedCommunitiesResponse {
        op: self.op_type().to_string(),
        communities: communities
      }
      )
      .unwrap()
  }
}

// impl Handler<Login> for ChatServer {

//   type Result = MessageResult<Login>;
//   fn handle(&mut self, msg: Login, _: &mut Context<Self>) -> Self::Result {

//     let conn = establish_connection();

//     // Fetch that username / email
//     let user: User_ = match User_::find_by_email_or_username(&conn, &msg.username_or_email) {
//       Ok(user) => user,
//       Err(_e) => return MessageResult(
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
//       Err(_e) => return MessageResult(
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
//       Err(_e) => return MessageResult(
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
//         CommunityResponse {
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
