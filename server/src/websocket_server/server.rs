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
use diesel::PgConnection;

use {Crud, Joinable, Likeable, Followable, Bannable, establish_connection, naive_now, naive_from_unix, SortType, has_slurs, remove_slurs};
use actions::community::*;
use actions::user::*;
use actions::post::*;
use actions::comment::*;
use actions::post_view::*;
use actions::comment_view::*;
use actions::category::*;
use actions::community_view::*;
use actions::user_view::*;
use actions::moderator_views::*;
use actions::moderator::*;

#[derive(EnumString,ToString,Debug)]
pub enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, EditCommunity, FollowCommunity, GetFollowedCommunities, GetUserDetails, GetModlog, BanFromCommunity, AddModToCommunity,
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
  sort: String,
  limit: Option<i64>,
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
  creator_id: i32,
  post_id: i32,
  removed: Option<bool>,
  reason: Option<String>,
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
  creator_id: i32,
  community_id: i32,
  name: String,
  url: Option<String>,
  body: Option<String>,
  removed: Option<bool>,
  reason: Option<String>,
  locked: Option<bool>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct EditCommunity {
  edit_id: i32,
  name: String,
  title: String,
  description: Option<String>,
  category_id: i32,
  removed: Option<bool>,
  reason: Option<String>,
  expires: Option<i64>,
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

#[derive(Serialize, Deserialize)]
pub struct GetUserDetails {
  user_id: i32,
  sort: String,
  limit: i64,
  community_id: Option<i32>,
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct GetUserDetailsResponse {
  op: String,
  user: UserView,
  follows: Vec<CommunityFollowerView>,
  moderates: Vec<CommunityModeratorView>,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
  saved_posts: Vec<PostView>,
  saved_comments: Vec<CommentView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetModlog {
  mod_user_id: Option<i32>,
  community_id: Option<i32>,
  limit: Option<i64>,
  page: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct GetModlogResponse {
  op: String,
  removed_posts: Vec<ModRemovePostView>,
  locked_posts: Vec<ModLockPostView>,
  removed_comments: Vec<ModRemoveCommentView>,
  removed_communities: Vec<ModRemoveCommunityView>,
  banned_from_community: Vec<ModBanFromCommunityView>,
  banned: Vec<ModBanView>,
  added_to_community: Vec<ModAddCommunityView>,
  added: Vec<ModAddView>,
}

#[derive(Serialize, Deserialize)]
pub struct BanFromCommunity {
  community_id: i32,
  user_id: i32,
  ban: bool,
  reason: Option<String>,
  expires: Option<i64>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct BanFromCommunityResponse {
  op: String,
  user: UserView,
  banned: bool,
}


#[derive(Serialize, Deserialize)]
pub struct AddModToCommunity {
  community_id: i32,
  user_id: i32,
  added: bool,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunityResponse {
  op: String,
  moderators: Vec<CommunityModeratorView>,
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

  fn send_community_message(&self, conn: &PgConnection, community_id: i32, message: &str, skip_id: usize) {
    let posts = PostView::list(conn,
                               PostListingType::Community, 
                               &SortType::New, 
                               Some(community_id), 
                               None,
                               None, 
                               999).unwrap();
    for post in posts {
      self.send_room_message(post.id, message, skip_id);
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
  }
}

/// Handler for Message message.
impl Handler<StandardMessage> for ChatServer {
  type Result = MessageResult<StandardMessage>;

  fn handle(&mut self, msg: StandardMessage, _: &mut Context<Self>) -> Self::Result {

    let json: Value = serde_json::from_str(&msg.msg)
      .expect("Couldn't parse message");

    let data = &json["data"].to_string();
    let op = &json["op"].as_str().unwrap();
    let user_operation: UserOperation = UserOperation::from_str(&op).unwrap();


    // TODO figure out how to do proper error handling here, instead of just returning
    // error strings
    let res: String = match user_operation {
      UserOperation::Login => {
        let login: Login = serde_json::from_str(data).unwrap();
        login.perform(self, msg.id)
      },
      UserOperation::Register => {
        let register: Register = serde_json::from_str(data).unwrap();
        register.perform(self, msg.id)
      },
      UserOperation::CreateCommunity => {
        let create_community: CreateCommunity = serde_json::from_str(data).unwrap();
        create_community.perform(self, msg.id)
      },
      UserOperation::ListCommunities => {
        let list_communities: ListCommunities = serde_json::from_str(data).unwrap();
        list_communities.perform(self, msg.id)
      },
      UserOperation::ListCategories => {
        let list_categories: ListCategories = ListCategories;
        list_categories.perform(self, msg.id)
      },
      UserOperation::CreatePost => {
        let create_post: CreatePost = serde_json::from_str(data).unwrap();
        create_post.perform(self, msg.id)
      },
      UserOperation::GetPost => {
        let get_post: GetPost = serde_json::from_str(data).unwrap();
        get_post.perform(self, msg.id)
      },
      UserOperation::GetCommunity => {
        let get_community: GetCommunity = serde_json::from_str(data).unwrap();
        get_community.perform(self, msg.id)
      },
      UserOperation::CreateComment => {
        let create_comment: CreateComment = serde_json::from_str(data).unwrap();
        create_comment.perform(self, msg.id)
      },
      UserOperation::EditComment => {
        let edit_comment: EditComment = serde_json::from_str(data).unwrap();
        edit_comment.perform(self, msg.id)
      },
      UserOperation::CreateCommentLike => {
        let create_comment_like: CreateCommentLike = serde_json::from_str(data).unwrap();
        create_comment_like.perform(self, msg.id)
      },
      UserOperation::GetPosts => {
        let get_posts: GetPosts = serde_json::from_str(data).unwrap();
        get_posts.perform(self, msg.id)
      },
      UserOperation::CreatePostLike => {
        let create_post_like: CreatePostLike = serde_json::from_str(data).unwrap();
        create_post_like.perform(self, msg.id)
      },
      UserOperation::EditPost => {
        let edit_post: EditPost = serde_json::from_str(data).unwrap();
        edit_post.perform(self, msg.id)
      },
      UserOperation::EditCommunity => {
        let edit_community: EditCommunity = serde_json::from_str(data).unwrap();
        edit_community.perform(self, msg.id)
      },
      UserOperation::FollowCommunity => {
        let follow_community: FollowCommunity = serde_json::from_str(data).unwrap();
        follow_community.perform(self, msg.id)
      },
      UserOperation::GetFollowedCommunities => {
        let followed_communities: GetFollowedCommunities = serde_json::from_str(data).unwrap();
        followed_communities.perform(self, msg.id)
      },
      UserOperation::GetUserDetails => {
        let get_user_details: GetUserDetails = serde_json::from_str(data).unwrap();
        get_user_details.perform(self, msg.id)
      },
      UserOperation::GetModlog => {
        let get_modlog: GetModlog = serde_json::from_str(data).unwrap();
        get_modlog.perform(self, msg.id)
      },
      UserOperation::BanFromCommunity => {
        let ban_from_community: BanFromCommunity = serde_json::from_str(data).unwrap();
        ban_from_community.perform(self, msg.id)
      },
      UserOperation::AddModToCommunity => {
        let mod_add_to_community: AddModToCommunity = serde_json::from_str(data).unwrap();
        mod_add_to_community.perform(self, msg.id)
      },
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

    if has_slurs(&self.username) {
      return self.error("No slurs");
    }

    // Register the new user
    let user_form = UserForm {
      name: self.username.to_owned(),
      fedi_name: "rrf".into(),
      email: self.email.to_owned(),
      password_encrypted: self.password.to_owned(),
      preferred_username: None,
      updated: None,
      admin: None,
      banned: None,
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

    if has_slurs(&self.name) || 
      has_slurs(&self.title) || 
      (self.description.is_some() && has_slurs(&self.description.to_owned().unwrap())) {
      return self.error("No slurs");
    }

    let user_id = claims.id;

    // When you create a community, make sure the user becomes a moderator and a follower

    let community_form = CommunityForm {
      name: self.name.to_owned(),
      title: self.title.to_owned(),
      description: self.description.to_owned(),
      category_id: self.category_id,
      creator_id: user_id,
      removed: None,
      updated: None,
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

    let sort = SortType::from_str(&self.sort).expect("listing sort");

    let communities: Vec<CommunityView> = CommunityView::list(&conn, user_id, sort, self.limit).unwrap();

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

    if has_slurs(&self.name) || 
      (self.body.is_some() && has_slurs(&self.body.to_owned().unwrap())) {
      return self.error("No slurs");
    }

    let user_id = claims.id;

    // Check for a ban
    if CommunityUserBanView::get(&conn, user_id, self.community_id).is_ok() {
      return self.error("You have been banned from this community");
    }

    let post_form = PostForm {
      name: self.name.to_owned(),
      url: self.url.to_owned(),
      body: self.body.to_owned(),
      community_id: self.community_id,
      creator_id: user_id,
      removed: None,
      locked: None,
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

    let comments = CommentView::list(&conn, &SortType::New, Some(self.id), None, user_id, 999).unwrap();

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

    // Check for a ban
    let post = Post::read(&conn, self.post_id).unwrap();
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return self.error("You have been banned from this community");
    }

    let content_slurs_removed = remove_slurs(&self.content.to_owned());

    let comment_form = CommentForm {
      content: content_slurs_removed,
      parent_id: self.parent_id.to_owned(),
      post_id: self.post_id,
      creator_id: user_id,
      removed: None,
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


    // Verify its the creator or a mod
    let orig_comment = CommentView::read(&conn, self.edit_id, None).unwrap();
    let mut editors: Vec<i32> = CommunityModeratorView::for_community(&conn, orig_comment.community_id)
      .unwrap()
      .into_iter()
      .map(|m| m.user_id)
      .collect();
    editors.push(self.creator_id);
    if !editors.contains(&user_id) {
      return self.error("Not allowed to edit comment.");
    }

    // Check for a ban
    if CommunityUserBanView::get(&conn, user_id, orig_comment.community_id).is_ok() {
      return self.error("You have been banned from this community");
    }

    let content_slurs_removed = remove_slurs(&self.content.to_owned());

    let comment_form = CommentForm {
      content: content_slurs_removed,
      parent_id: self.parent_id,
      post_id: self.post_id,
      creator_id: self.creator_id,
      removed: self.removed.to_owned(),
      updated: Some(naive_now())
    };

    let _updated_comment = match Comment::update(&conn, self.edit_id, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => {
        return self.error("Couldn't update Comment");
      }
    };

    // Mod tables
    if let Some(removed) = self.removed.to_owned() {
      let form = ModRemoveCommentForm {
        mod_user_id: user_id,
        comment_id: self.edit_id,
        removed: Some(removed),
        reason: self.reason.to_owned(),
      };
      ModRemoveComment::create(&conn, &form).unwrap();
    }


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

    // Check for a ban
    let post = Post::read(&conn, self.post_id).unwrap();
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return self.error("You have been banned from this community");
    }

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

    let type_ = PostListingType::from_str(&self.type_).expect("listing type");
    let sort = SortType::from_str(&self.sort).expect("listing sort");

    let posts = match PostView::list(&conn, type_, &sort, self.community_id, None, user_id, self.limit) {
      Ok(posts) => posts,
      Err(_e) => {
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

    // Check for a ban
    let post = Post::read(&conn, self.post_id).unwrap();
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return self.error("You have been banned from this community");
    }

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

    if has_slurs(&self.name) || 
      (self.body.is_some() && has_slurs(&self.body.to_owned().unwrap())) {
      return self.error("No slurs");
    }

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return self.error("Not logged in.");
      }
    };

    let user_id = claims.id;

    // Verify its the creator or a mod
    let mut editors: Vec<i32> = CommunityModeratorView::for_community(&conn, self.community_id)
      .unwrap()
      .into_iter()
      .map(|m| m.user_id)
      .collect();
    editors.push(self.creator_id);
    if !editors.contains(&user_id) {
      return self.error("Not allowed to edit comment.");
    }

    // Check for a ban
    if CommunityUserBanView::get(&conn, user_id, self.community_id).is_ok() {
      return self.error("You have been banned from this community");
    }

    let post_form = PostForm {
      name: self.name.to_owned(),
      url: self.url.to_owned(),
      body: self.body.to_owned(),
      creator_id: self.creator_id.to_owned(),
      community_id: self.community_id,
      removed: self.removed.to_owned(),
      locked: self.locked.to_owned(),
      updated: Some(naive_now())
    };

    let _updated_post = match Post::update(&conn, self.edit_id, &post_form) {
      Ok(post) => post,
      Err(_e) => {
        return self.error("Couldn't update Post");
      }
    };

    // Mod tables
    if let Some(removed) = self.removed.to_owned() {
      let form = ModRemovePostForm {
        mod_user_id: user_id,
        post_id: self.edit_id,
        removed: Some(removed),
        reason: self.reason.to_owned(),
      };
      ModRemovePost::create(&conn, &form).unwrap();
    }

    if let Some(locked) = self.locked.to_owned() {
      let form = ModLockPostForm {
        mod_user_id: user_id,
        post_id: self.edit_id,
        locked: Some(locked),
      };
      ModLockPost::create(&conn, &form).unwrap();
    }

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

    if has_slurs(&self.name) || has_slurs(&self.title) {
      return self.error("No slurs");
    }

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
      removed: self.removed.to_owned(),
      updated: Some(naive_now())
    };

    let _updated_community = match Community::update(&conn, self.edit_id, &community_form) {
      Ok(community) => community,
      Err(_e) => {
        return self.error("Couldn't update Community");
      }
    };

    // Mod tables
    if let Some(removed) = self.removed.to_owned() {
      let expires = match self.expires {
        Some(time) => Some(naive_from_unix(time)),
        None => None
      };
      let form = ModRemoveCommunityForm {
        mod_user_id: user_id,
        community_id: self.edit_id,
        removed: Some(removed),
        reason: self.reason.to_owned(),
        expires: expires
      };
      ModRemoveCommunity::create(&conn, &form).unwrap();
    }

    let community_view = CommunityView::read(&conn, self.edit_id, Some(user_id)).unwrap();

    let community_out = serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view
      }
      )
      .unwrap();

    let community_view_sent = CommunityView::read(&conn, self.edit_id, None).unwrap();

    let community_sent = serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view_sent
      }
      )
      .unwrap();

    chat.send_community_message(&conn, self.edit_id, &community_sent, addr);

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

impl Perform for GetUserDetails {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetUserDetails
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


    //TODO add save
    let sort = SortType::from_str(&self.sort).expect("listing sort");

    let user_view = UserView::read(&conn, self.user_id).unwrap();
    let posts = PostView::list(&conn, PostListingType::All, &sort, self.community_id, Some(self.user_id), user_id, self.limit).unwrap();
    let comments = CommentView::list(&conn, &sort, None, Some(self.user_id), user_id, self.limit).unwrap();
    let follows = CommunityFollowerView::for_user(&conn, self.user_id).unwrap();
    let moderates = CommunityModeratorView::for_user(&conn, self.user_id).unwrap();

    // Return the jwt
    serde_json::to_string(
      &GetUserDetailsResponse {
        op: self.op_type().to_string(),
        user: user_view,
        follows: follows,
        moderates: moderates, 
        comments: comments,
        posts: posts,
        saved_posts: Vec::new(),
        saved_comments: Vec::new(),
      }
      )
      .unwrap()
  }
}

impl Perform for GetModlog {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetModlog
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> String {

    let conn = establish_connection();

    let removed_posts = ModRemovePostView::list(&conn, self.community_id, self.mod_user_id, self.limit, self.page).unwrap();
    let locked_posts = ModLockPostView::list(&conn, self.community_id, self.mod_user_id, self.limit, self.page).unwrap();
    let removed_comments = ModRemoveCommentView::list(&conn, self.community_id, self.mod_user_id, self.limit, self.page).unwrap();
    let removed_communities = ModRemoveCommunityView::list(&conn, self.mod_user_id, self.limit, self.page).unwrap();
    let banned_from_community = ModBanFromCommunityView::list(&conn, self.community_id, self.mod_user_id, self.limit, self.page).unwrap();
    let banned = ModBanView::list(&conn, self.mod_user_id, self.limit, self.page).unwrap();
    let added_to_community = ModAddCommunityView::list(&conn, self.community_id, self.mod_user_id, self.limit, self.page).unwrap();
    let added = ModAddView::list(&conn, self.mod_user_id, self.limit, self.page).unwrap();

    // Return the jwt
    serde_json::to_string(
      &GetModlogResponse {
        op: self.op_type().to_string(),
        removed_posts: removed_posts,
        locked_posts: locked_posts,
        removed_comments: removed_comments,
        removed_communities: removed_communities,
        banned_from_community: banned_from_community,
        banned: banned,
        added_to_community: added_to_community,
        added: added,
      }
      )
      .unwrap()
  }
}

impl Perform for BanFromCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::BanFromCommunity
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

    let community_user_ban_form = CommunityUserBanForm {
      community_id: self.community_id,
      user_id: self.user_id,
    };

    if self.ban {
      match CommunityUserBan::ban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => {
          return self.error("Community user ban already exists");
        }
      };
    } else {
      match CommunityUserBan::unban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => {
          return self.error("Community user ban already exists");
        }
      };
    }

    // Mod tables
    let expires = match self.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None
    };

    let form = ModBanFromCommunityForm {
      mod_user_id: user_id,
      other_user_id: self.user_id,
      community_id: self.community_id,
      reason: self.reason.to_owned(),
      banned: Some(self.ban),
      expires: expires,
    };
    ModBanFromCommunity::create(&conn, &form).unwrap();

    let user_view = UserView::read(&conn, self.user_id).unwrap();

    let res = serde_json::to_string(
      &BanFromCommunityResponse {
        op: self.op_type().to_string(), 
        user: user_view,
        banned: self.ban
      }
      )
      .unwrap();


    chat.send_community_message(&conn, self.community_id, &res, addr);

    res
  }
}

impl Perform for AddModToCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::AddModToCommunity
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

    let community_moderator_form = CommunityModeratorForm {
      community_id: self.community_id,
      user_id: self.user_id
    };

    if self.added {
      match CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return self.error("Community moderator already exists.");
        }
      };
    } else {
      match CommunityModerator::leave(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return self.error("Community moderator already exists.");
        }
      };
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: self.user_id,
      community_id: self.community_id,
      removed: Some(!self.added),
    };
    ModAddCommunity::create(&conn, &form).unwrap();

    let moderators = CommunityModeratorView::for_community(&conn, self.community_id).unwrap();

    let res = serde_json::to_string(
      &AddModToCommunityResponse {
        op: self.op_type().to_string(), 
        moderators: moderators,
      }
      )
      .unwrap();


    chat.send_community_message(&conn, self.community_id, &res, addr);

    res

  }
}

