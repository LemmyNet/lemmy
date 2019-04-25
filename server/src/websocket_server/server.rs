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
use failure::Error;

use {Crud, Joinable, Likeable, Followable, Bannable, Saveable, establish_connection, naive_now, naive_from_unix, SortType, SearchType, has_slurs, remove_slurs};
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
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, SaveComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, SavePost, EditCommunity, FollowCommunity, GetFollowedCommunities, GetUserDetails, GetReplies, GetModlog, BanFromCommunity, AddModToCommunity, CreateSite, EditSite, GetSite, AddAdmin, BanUser, Search
}

#[derive(Fail, Debug)]
#[fail(display = "{{\"op\":\"{}\", \"error\":\"{}\"}}", op, message)]
pub struct ErrorMessage {
  op: String,
  message: String
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
  password_verify: String,
  admin: bool,
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
  page: Option<i64>,
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
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPosts {
  type_: String,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
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
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
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
  read: Option<bool>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct SaveComment {
  comment_id: i32,
  save: bool,
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
  locked: Option<bool>,
  reason: Option<String>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct SavePost {
  post_id: i32,
  save: bool,
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
  page: Option<i64>,
  limit: Option<i64>,
  community_id: Option<i32>,
  saved_only: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserDetailsResponse {
  op: String,
  user: UserView,
  follows: Vec<CommunityFollowerView>,
  moderates: Vec<CommunityModeratorView>,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetModlog {
  mod_user_id: Option<i32>,
  community_id: Option<i32>,
  page: Option<i64>,
  limit: Option<i64>,
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

#[derive(Serialize, Deserialize)]
pub struct CreateSite {
  name: String,
  description: Option<String>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct EditSite {
  name: String,
  description: Option<String>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct GetSite {
}

#[derive(Serialize, Deserialize)]
pub struct SiteResponse {
  op: String,
  site: SiteView,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteResponse {
  op: String,
  site: Option<SiteView>,
  admins: Vec<UserView>,
  banned: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct AddAdmin {
  user_id: i32,
  added: bool,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct AddAdminResponse {
  op: String,
  admins: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct BanUser {
  user_id: i32,
  ban: bool,
  reason: Option<String>,
  expires: Option<i64>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct BanUserResponse {
  op: String,
  user: UserView,
  banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetReplies {
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  unread_only: bool,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct GetRepliesResponse {
  op: String,
  replies: Vec<ReplyView>,
}

#[derive(Serialize, Deserialize)]
pub struct Search {
  q: String,
  type_: String,
  community_id: Option<i32>,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
  op: String,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
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

  fn send_community_message(&self, conn: &PgConnection, community_id: i32, message: &str, skip_id: usize) -> Result<(), Error> {
    let posts = PostView::list(conn,
                               PostListingType::Community, 
                               &SortType::New, 
                               Some(community_id), 
                               None,
                               None, 
                               None,
                               false,
                               false,
                               None,
                               Some(9999))?;
    for post in posts {
      self.send_room_message(post.id, message, skip_id);
    }

    Ok(())
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

    let msg_out = match parse_json_message(self, msg) {
      Ok(m) => m,
      Err(e) => e.to_string()
    };

    MessageResult(msg_out)
  }
}

fn parse_json_message(chat: &mut ChatServer, msg: StandardMessage) -> Result<String, Error> {

  let json: Value = serde_json::from_str(&msg.msg)?;
  let data = &json["data"].to_string();
  let op = &json["op"].as_str().unwrap();

  let user_operation: UserOperation = UserOperation::from_str(&op)?;

  match user_operation {
    UserOperation::Login => {
      let login: Login = serde_json::from_str(data)?;
      login.perform(chat, msg.id)
    },
    UserOperation::Register => {
      let register: Register = serde_json::from_str(data)?;
      register.perform(chat, msg.id)
    },
    UserOperation::CreateCommunity => {
      let create_community: CreateCommunity = serde_json::from_str(data)?;
      create_community.perform(chat, msg.id)
    },
    UserOperation::ListCommunities => {
      let list_communities: ListCommunities = serde_json::from_str(data)?;
      list_communities.perform(chat, msg.id)
    },
    UserOperation::ListCategories => {
      let list_categories: ListCategories = ListCategories;
      list_categories.perform(chat, msg.id)
    },
    UserOperation::CreatePost => {
      let create_post: CreatePost = serde_json::from_str(data)?;
      create_post.perform(chat, msg.id)
    },
    UserOperation::GetPost => {
      let get_post: GetPost = serde_json::from_str(data)?;
      get_post.perform(chat, msg.id)
    },
    UserOperation::GetCommunity => {
      let get_community: GetCommunity = serde_json::from_str(data)?;
      get_community.perform(chat, msg.id)
    },
    UserOperation::CreateComment => {
      let create_comment: CreateComment = serde_json::from_str(data)?;
      create_comment.perform(chat, msg.id)
    },
    UserOperation::EditComment => {
      let edit_comment: EditComment = serde_json::from_str(data)?;
      edit_comment.perform(chat, msg.id)
    },
    UserOperation::SaveComment => {
      let save_post: SaveComment = serde_json::from_str(data)?;
      save_post.perform(chat, msg.id)
    },
    UserOperation::CreateCommentLike => {
      let create_comment_like: CreateCommentLike = serde_json::from_str(data)?;
      create_comment_like.perform(chat, msg.id)
    },
    UserOperation::GetPosts => {
      let get_posts: GetPosts = serde_json::from_str(data)?;
      get_posts.perform(chat, msg.id)
    },
    UserOperation::CreatePostLike => {
      let create_post_like: CreatePostLike = serde_json::from_str(data)?;
      create_post_like.perform(chat, msg.id)
    },
    UserOperation::EditPost => {
      let edit_post: EditPost = serde_json::from_str(data)?;
      edit_post.perform(chat, msg.id)
    },
    UserOperation::SavePost => {
      let save_post: SavePost = serde_json::from_str(data)?;
      save_post.perform(chat, msg.id)
    },
    UserOperation::EditCommunity => {
      let edit_community: EditCommunity = serde_json::from_str(data)?;
      edit_community.perform(chat, msg.id)
    },
    UserOperation::FollowCommunity => {
      let follow_community: FollowCommunity = serde_json::from_str(data)?;
      follow_community.perform(chat, msg.id)
    },
    UserOperation::GetFollowedCommunities => {
      let followed_communities: GetFollowedCommunities = serde_json::from_str(data)?;
      followed_communities.perform(chat, msg.id)
    },
    UserOperation::GetUserDetails => {
      let get_user_details: GetUserDetails = serde_json::from_str(data)?;
      get_user_details.perform(chat, msg.id)
    },
    UserOperation::GetModlog => {
      let get_modlog: GetModlog = serde_json::from_str(data)?;
      get_modlog.perform(chat, msg.id)
    },
    UserOperation::BanFromCommunity => {
      let ban_from_community: BanFromCommunity = serde_json::from_str(data)?;
      ban_from_community.perform(chat, msg.id)
    },
    UserOperation::AddModToCommunity => {
      let mod_add_to_community: AddModToCommunity = serde_json::from_str(data)?;
      mod_add_to_community.perform(chat, msg.id)
    },
    UserOperation::CreateSite => {
      let create_site: CreateSite = serde_json::from_str(data)?;
      create_site.perform(chat, msg.id)
    },
    UserOperation::EditSite => {
      let edit_site: EditSite = serde_json::from_str(data)?;
      edit_site.perform(chat, msg.id)
    },
    UserOperation::GetSite => {
      let get_site: GetSite = serde_json::from_str(data)?;
      get_site.perform(chat, msg.id)
    },
    UserOperation::AddAdmin => {
      let add_admin: AddAdmin = serde_json::from_str(data)?;
      add_admin.perform(chat, msg.id)
    },
    UserOperation::BanUser => {
      let ban_user: BanUser = serde_json::from_str(data)?;
      ban_user.perform(chat, msg.id)
    },
    UserOperation::GetReplies => {
      let get_replies: GetReplies = serde_json::from_str(data)?;
      get_replies.perform(chat, msg.id)
    },
    UserOperation::Search => {
      let search: Search = serde_json::from_str(data)?;
      search.perform(chat, msg.id)
    },
  }
}

pub trait Perform {
  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error>;
  fn op_type(&self) -> UserOperation;
  fn error(&self, error_msg: &str) -> ErrorMessage {
    ErrorMessage {
      op: self.op_type().to_string(), 
      message: error_msg.to_string()
    }
  }
}

impl Perform for Login {

  fn op_type(&self) -> UserOperation {
    UserOperation::Login
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    // Fetch that username / email
    let user: User_ = match User_::find_by_email_or_username(&conn, &self.username_or_email) {
      Ok(user) => user,
      Err(_e) => return Err(self.error("Couldn't find that username or email"))?
    };

    // Verify the password
    let valid: bool = verify(&self.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(self.error("Password incorrect"))?
    }

    // Return the jwt
    Ok(
      serde_json::to_string(
        &LoginResponse {
          op: self.op_type().to_string(),
          jwt: user.jwt()
        }
        )?
      )
  }

}

impl Perform for Register {
  fn op_type(&self) -> UserOperation {
    UserOperation::Register
  }
  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    // Make sure passwords match
    if &self.password != &self.password_verify {
      return Err(self.error("Passwords do not match."))?
    }

    if has_slurs(&self.username) {
      return Err(self.error("No slurs"))?
    }

    // Make sure there are no admins
    if self.admin && UserView::admins(&conn)?.len() > 0 {
      return Err(self.error("Sorry, there's already an admin."))?
    }

    // Register the new user
    let user_form = UserForm {
      name: self.username.to_owned(),
      fedi_name: "rrf".into(),
      email: self.email.to_owned(),
      password_encrypted: self.password.to_owned(),
      preferred_username: None,
      updated: None,
      admin: self.admin,
      banned: false,
    };

    // Create the user
    let inserted_user = match User_::register(&conn, &user_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(self.error("User already exists."))?
      }
    };

    // If its an admin, add them as a mod and follower to main
    if self.admin {
      let community_moderator_form = CommunityModeratorForm {
        community_id: 1,
        user_id: inserted_user.id,
      };

      let _inserted_community_moderator = match CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(self.error("Community moderator already exists."))?
        }
      };

      let community_follower_form = CommunityFollowerForm {
        community_id: 1,
        user_id: inserted_user.id,
      };

      let _inserted_community_follower = match CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(self.error("Community follower already exists."))?
        }
      };
    }


    // Return the jwt
    Ok(
      serde_json::to_string(
        &LoginResponse {
          op: self.op_type().to_string(), 
          jwt: inserted_user.jwt()
        }
        )?
      )

  }
}

impl Perform for CreateCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateCommunity
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    if has_slurs(&self.name) || 
      has_slurs(&self.title) || 
        (self.description.is_some() && has_slurs(&self.description.to_owned().unwrap())) {
          return Err(self.error("No slurs"))?
        }

    let user_id = claims.id;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
    }

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
        return Err(self.error("Community already exists."))?
      }
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let _inserted_community_moderator = match CommunityModerator::join(&conn, &community_moderator_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(self.error("Community moderator already exists."))?
      }
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let _inserted_community_follower = match CommunityFollower::follow(&conn, &community_follower_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(self.error("Community follower already exists."))?
      }
    };

    let community_view = CommunityView::read(&conn, inserted_community.id, Some(user_id))?;

    Ok(
      serde_json::to_string(
        &CommunityResponse {
          op: self.op_type().to_string(), 
          community: community_view
        }
        )?
      )
  }
}

impl Perform for ListCommunities {
  fn op_type(&self) -> UserOperation {
    UserOperation::ListCommunities
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

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

    let sort = SortType::from_str(&self.sort)?;

    let communities: Vec<CommunityView> = CommunityView::list(&conn, user_id, sort, self.page, self.limit)?;

    // Return the jwt
    Ok(
      serde_json::to_string(
        &ListCommunitiesResponse {
          op: self.op_type().to_string(),
          communities: communities
        }
        )?
      )
  }
}

impl Perform for ListCategories {
  fn op_type(&self) -> UserOperation {
    UserOperation::ListCategories
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let categories: Vec<Category> = Category::list_all(&conn)?;

    // Return the jwt
    Ok(
      serde_json::to_string(
        &ListCategoriesResponse {
          op: self.op_type().to_string(),
          categories: categories
        }
        )?
      )
  }
}

impl Perform for CreatePost {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreatePost
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    if has_slurs(&self.name) || 
      (self.body.is_some() && has_slurs(&self.body.to_owned().unwrap())) {
        return Err(self.error("No slurs"))?
      }

    let user_id = claims.id;

    // Check for a community ban
    if CommunityUserBanView::get(&conn, user_id, self.community_id).is_ok() {
      return Err(self.error("You have been banned from this community"))?
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
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
        return Err(self.error("Couldn't create Post"))?
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
        return Err(self.error("Couldn't like post."))?
      }
    };

    // Refetch the view
    let post_view = match PostView::read(&conn, inserted_post.id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => {
        return Err(self.error("Couldn't find Post"))?
      }
    };

    Ok(
      serde_json::to_string(
        &PostResponse {
          op: self.op_type().to_string(), 
          post: post_view
        }
        )?
      )
  }
}


impl Perform for GetPost {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetPost
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

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
        return Err(self.error("Couldn't find Post"))?
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

    let comments = CommentView::list(&conn, &SortType::New, Some(self.id), None, None, user_id, false, None, Some(9999))?;

    let community = CommunityView::read(&conn, post_view.community_id, user_id)?;

    let moderators = CommunityModeratorView::for_community(&conn, post_view.community_id)?;

    let admins = UserView::admins(&conn)?;

    // Return the jwt
    Ok(
      serde_json::to_string(
        &GetPostResponse {
          op: self.op_type().to_string(),
          post: post_view,
          comments: comments,
          community: community,
          moderators: moderators,
          admins: admins,
        }
        )?
      )
  }
}

impl Perform for GetCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetCommunity
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

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
        return Err(self.error("Couldn't find Community"))?
      }
    };

    let moderators = match CommunityModeratorView::for_community(&conn, self.id) {
      Ok(moderators) => moderators,
      Err(_e) => {
        return Err(self.error("Couldn't find Community"))?
      }
    };

    let admins = UserView::admins(&conn)?;

    // Return the jwt
    Ok(
      serde_json::to_string(
        &GetCommunityResponse {
          op: self.op_type().to_string(),
          community: community_view,
          moderators: moderators,
          admins: admins,
        }
        )?
      )
  }
}

impl Perform for CreateComment {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateComment
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Check for a community ban
    let post = Post::read(&conn, self.post_id)?;
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(self.error("You have been banned from this community"))?
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
    }

    let content_slurs_removed = remove_slurs(&self.content.to_owned());

    let comment_form = CommentForm {
      content: content_slurs_removed,
      parent_id: self.parent_id.to_owned(),
      post_id: self.post_id,
      creator_id: user_id,
      removed: None,
      read: None,
      updated: None
    };

    let inserted_comment = match Comment::create(&conn, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => {
        return Err(self.error("Couldn't create Comment"))?
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
        return Err(self.error("Couldn't like comment."))?
      }
    };

    let comment_view = CommentView::read(&conn, inserted_comment.id, Some(user_id))?;

    let mut comment_sent = comment_view.clone();
    comment_sent.my_vote = None;
    comment_sent.user_id = None;

    let comment_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )?;

    let comment_sent_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_sent
      }
      )?;

    chat.send_room_message(self.post_id, &comment_sent_out, addr);

    Ok(comment_out)
  }
}

impl Perform for EditComment {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditComment
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;


    // You are allowed to mark the comment as read even if you're banned.
    if self.read.is_none() {

      // Verify its the creator or a mod, or an admin
      let orig_comment = CommentView::read(&conn, self.edit_id, None)?;
      let mut editors: Vec<i32> = vec![self.creator_id];
      editors.append(
        &mut CommunityModeratorView::for_community(&conn, orig_comment.community_id)
        ?
        .into_iter()
        .map(|m| m.user_id)
        .collect()
        );
      editors.append(
        &mut UserView::admins(&conn)
        ?
        .into_iter()
        .map(|a| a.id)
        .collect()
        );

      if !editors.contains(&user_id) {
        return Err(self.error("Not allowed to edit comment."))?
      }

      // Check for a community ban
      if CommunityUserBanView::get(&conn, user_id, orig_comment.community_id).is_ok() {
        return Err(self.error("You have been banned from this community"))?
      }

      // Check for a site ban
      if UserView::read(&conn, user_id)?.banned {
        return Err(self.error("You have been banned from the site"))?
      }

    }

    let content_slurs_removed = remove_slurs(&self.content.to_owned());

    let comment_form = CommentForm {
      content: content_slurs_removed,
      parent_id: self.parent_id,
      post_id: self.post_id,
      creator_id: self.creator_id,
      removed: self.removed.to_owned(),
      read: self.read.to_owned(),
      updated: if self.read.is_some() { None } else {Some(naive_now())}
      };

    let _updated_comment = match Comment::update(&conn, self.edit_id, &comment_form) {
      Ok(comment) => comment,
      Err(_e) => {
        return Err(self.error("Couldn't update Comment"))?
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
      ModRemoveComment::create(&conn, &form)?;
    }


    let comment_view = CommentView::read(&conn, self.edit_id, Some(user_id))?;

    let mut comment_sent = comment_view.clone();
    comment_sent.my_vote = None;
    comment_sent.user_id = None;

    let comment_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )?;

    let comment_sent_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_sent
      }
      )?;

    chat.send_room_message(self.post_id, &comment_sent_out, addr);

    Ok(comment_out)
  }
}

impl Perform for SaveComment {
  fn op_type(&self) -> UserOperation {
    UserOperation::SaveComment
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    let comment_saved_form = CommentSavedForm {
      comment_id: self.comment_id,
      user_id: user_id,
    };

    if self.save {
      match CommentSaved::save(&conn, &comment_saved_form) {
        Ok(comment) => comment,
        Err(_e) => {
          return Err(self.error("Couldnt do comment save"))?
        }
      };
    } else {
      match CommentSaved::unsave(&conn, &comment_saved_form) {
        Ok(comment) => comment,
        Err(_e) => {
          return Err(self.error("Couldnt do comment save"))?
        }
      };
    }

    let comment_view = CommentView::read(&conn, self.comment_id, Some(user_id))?;

    let comment_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: comment_view
      }
      )
      ?;

    Ok(comment_out)
  }
}


impl Perform for CreateCommentLike {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateCommentLike
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Check for a community ban
    let post = Post::read(&conn, self.post_id)?;
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(self.error("You have been banned from this community"))?
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
    }

    let like_form = CommentLikeForm {
      comment_id: self.comment_id,
      post_id: self.post_id,
      user_id: user_id,
      score: self.score
    };

    // Remove any likes first
    CommentLike::remove(&conn, &like_form)?;

    // Only add the like if the score isnt 0
    if &like_form.score != &0 {
      let _inserted_like = match CommentLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => {
          return Err(self.error("Couldn't like comment."))?
        }
      };
    }

    // Have to refetch the comment to get the current state
    let liked_comment = CommentView::read(&conn, self.comment_id, Some(user_id))?;

    let mut liked_comment_sent = liked_comment.clone();
    liked_comment_sent.my_vote = None;
    liked_comment_sent.user_id = None;

    let like_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: liked_comment
      }
      )?;

    let like_sent_out = serde_json::to_string(
      &CommentResponse {
        op: self.op_type().to_string(), 
        comment: liked_comment_sent
      }
      )?;

    chat.send_room_message(self.post_id, &like_sent_out, addr);

    Ok(like_out)
  }
}


impl Perform for GetPosts {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetPosts
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

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

    let type_ = PostListingType::from_str(&self.type_)?;
    let sort = SortType::from_str(&self.sort)?;

    let posts = match PostView::list(&conn, 
                                     type_, 
                                     &sort, 
                                     self.community_id, 
                                     None,
                                     None,
                                     user_id, 
                                     false, 
                                     false, 
                                     self.page, 
                                     self.limit) {
      Ok(posts) => posts,
      Err(_e) => {
        return Err(self.error("Couldn't get posts"))?
      }
    };

    // Return the jwt
    Ok(
      serde_json::to_string(
        &GetPostsResponse {
          op: self.op_type().to_string(),
          posts: posts
        }
        )?
      )
  }
}


impl Perform for CreatePostLike {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreatePostLike
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Check for a community ban
    let post = Post::read(&conn, self.post_id)?;
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(self.error("You have been banned from this community"))?
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
    }

    let like_form = PostLikeForm {
      post_id: self.post_id,
      user_id: user_id,
      score: self.score
    };

    // Remove any likes first
    PostLike::remove(&conn, &like_form)?;

    // Only add the like if the score isnt 0
    if &like_form.score != &0 {
      let _inserted_like = match PostLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => {
          return Err(self.error("Couldn't like post."))?
        }
      };
    }

    let post_view = match PostView::read(&conn, self.post_id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => {
        return Err(self.error("Couldn't find Post"))?
      }
    };

    // just output the score

    let like_out = serde_json::to_string(
      &CreatePostLikeResponse {
        op: self.op_type().to_string(), 
        post: post_view
      }
      )?;

    Ok(like_out)
  }
}

impl Perform for EditPost {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditPost
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    if has_slurs(&self.name) || 
      (self.body.is_some() && has_slurs(&self.body.to_owned().unwrap())) {
        return Err(self.error("No slurs"))?
      }

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Verify its the creator or a mod or admin
    let mut editors: Vec<i32> = vec![self.creator_id];
    editors.append(
      &mut CommunityModeratorView::for_community(&conn, self.community_id)
      ?
      .into_iter()
      .map(|m| m.user_id)
      .collect()
      );
    editors.append(
      &mut UserView::admins(&conn)
      ?
      .into_iter()
      .map(|a| a.id)
      .collect()
      );
    if !editors.contains(&user_id) {
      return Err(self.error("Not allowed to edit post."))?
    }

    // Check for a community ban
    if CommunityUserBanView::get(&conn, user_id, self.community_id).is_ok() {
      return Err(self.error("You have been banned from this community"))?
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
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
        return Err(self.error("Couldn't update Post"))?
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
      ModRemovePost::create(&conn, &form)?;
    }

    if let Some(locked) = self.locked.to_owned() {
      let form = ModLockPostForm {
        mod_user_id: user_id,
        post_id: self.edit_id,
        locked: Some(locked),
      };
      ModLockPost::create(&conn, &form)?;
    }

    let post_view = PostView::read(&conn, self.edit_id, Some(user_id))?;

    let mut post_sent = post_view.clone();
    post_sent.my_vote = None;

    let post_out = serde_json::to_string(
      &PostResponse {
        op: self.op_type().to_string(), 
        post: post_view
      }
      )
      ?;

    let post_sent_out = serde_json::to_string(
      &PostResponse {
        op: self.op_type().to_string(), 
        post: post_sent
      }
      )
      ?;

    chat.send_room_message(self.edit_id, &post_sent_out, addr);

    Ok(post_out)
  }
}

impl Perform for SavePost {
  fn op_type(&self) -> UserOperation {
    UserOperation::SavePost
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    let post_saved_form = PostSavedForm {
      post_id: self.post_id,
      user_id: user_id,
    };

    if self.save {
      match PostSaved::save(&conn, &post_saved_form) {
        Ok(post) => post,
        Err(_e) => {
          return Err(self.error("Couldnt do post save"))?
        }
      };
    } else {
      match PostSaved::unsave(&conn, &post_saved_form) {
        Ok(post) => post,
        Err(_e) => {
          return Err(self.error("Couldnt do post save"))?
        }
      };
    }

    let post_view = PostView::read(&conn, self.post_id, Some(user_id))?;

    let post_out = serde_json::to_string(
      &PostResponse {
        op: self.op_type().to_string(), 
        post: post_view
      }
      )
      ?;

    Ok(post_out)
  }
}

impl Perform for EditCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditCommunity
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    if has_slurs(&self.name) || has_slurs(&self.title) {
      return Err(self.error("No slurs"))?
    }

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(self.error("You have been banned from the site"))?
    }

    // Verify its a mod
    let mut editors: Vec<i32> = Vec::new();
    editors.append(
      &mut CommunityModeratorView::for_community(&conn, self.edit_id)
      ?
      .into_iter()
      .map(|m| m.user_id)
      .collect()
      );
    editors.append(
      &mut UserView::admins(&conn)
      ?
      .into_iter()
      .map(|a| a.id)
      .collect()
      );
    if !editors.contains(&user_id) {
      return Err(self.error("Not allowed to edit community"))?
    }

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
        return Err(self.error("Couldn't update Community"))?
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
      ModRemoveCommunity::create(&conn, &form)?;
    }

    let community_view = CommunityView::read(&conn, self.edit_id, Some(user_id))?;

    let community_out = serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view
      }
      )
      ?;

    let community_view_sent = CommunityView::read(&conn, self.edit_id, None)?;

    let community_sent = serde_json::to_string(
      &CommunityResponse {
        op: self.op_type().to_string(), 
        community: community_view_sent
      }
      )
      ?;

    chat.send_community_message(&conn, self.edit_id, &community_sent, addr)?;

    Ok(community_out)
  }
}


impl Perform for FollowCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::FollowCommunity
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
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
          return Err(self.error("Community follower already exists."))?
        }
      };
    } else {
      match CommunityFollower::ignore(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(self.error("Community follower already exists."))?
        }
      };
    }

    let community_view = CommunityView::read(&conn, self.community_id, Some(user_id))?;

    Ok(
      serde_json::to_string(
        &CommunityResponse {
          op: self.op_type().to_string(), 
          community: community_view
        }
        )?
      )
  }
}

impl Perform for GetFollowedCommunities {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetFollowedCommunities
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    let communities: Vec<CommunityFollowerView> = match CommunityFollowerView::for_user(&conn, user_id) {
      Ok(communities) => communities,
      Err(_e) => {
        return Err(self.error("System error, try logging out and back in."))?
      }
    };

    // Return the jwt
    Ok(
      serde_json::to_string(
        &GetFollowedCommunitiesResponse {
          op: self.op_type().to_string(),
          communities: communities
        }
        )?
      )
  }
}

impl Perform for GetUserDetails {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetUserDetails
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    //TODO add save
    let sort = SortType::from_str(&self.sort)?;

    let user_view = UserView::read(&conn, self.user_id)?;
    // If its saved only, you don't care what creator it was
    let posts = if self.saved_only {
      PostView::list(&conn, 
                     PostListingType::All, 
                     &sort, 
                     self.community_id, 
                     None, 
                     None,
                     Some(self.user_id), 
                     self.saved_only, 
                     false, 
                     self.page, 
                     self.limit)?
    } else {
      PostView::list(&conn, 
                     PostListingType::All, 
                     &sort, 
                     self.community_id, 
                     Some(self.user_id), 
                     None, 
                     None, 
                     self.saved_only, 
                     false, 
                     self.page, 
                     self.limit)?
    };
    let comments = if self.saved_only {
      CommentView::list(&conn, 
                        &sort, 
                        None, 
                        None, 
                        None, 
                        Some(self.user_id), 
                        self.saved_only, 
                        self.page, 
                        self.limit)?
    } else {
      CommentView::list(&conn, 
                        &sort, 
                        None, 
                        Some(self.user_id), 
                        None, 
                        None, 
                        self.saved_only, 
                        self.page, 
                        self.limit)?
    };

    let follows = CommunityFollowerView::for_user(&conn, self.user_id)?;
    let moderates = CommunityModeratorView::for_user(&conn, self.user_id)?;

    // Return the jwt
    Ok(
      serde_json::to_string(
        &GetUserDetailsResponse {
          op: self.op_type().to_string(),
          user: user_view,
          follows: follows,
          moderates: moderates, 
          comments: comments,
          posts: posts,
        }
        )?
      )
  }
}

impl Perform for GetModlog {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetModlog
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let removed_posts = ModRemovePostView::list(&conn, self.community_id, self.mod_user_id, self.page, self.limit)?;
    let locked_posts = ModLockPostView::list(&conn, self.community_id, self.mod_user_id, self.page, self.limit)?;
    let removed_comments = ModRemoveCommentView::list(&conn, self.community_id, self.mod_user_id, self.page, self.limit)?;
    let banned_from_community = ModBanFromCommunityView::list(&conn, self.community_id, self.mod_user_id, self.page, self.limit)?;
    let added_to_community = ModAddCommunityView::list(&conn, self.community_id, self.mod_user_id, self.page, self.limit)?;

    // These arrays are only for the full modlog, when a community isn't given
    let mut removed_communities = Vec::new();
    let mut banned = Vec::new();
    let mut added = Vec::new();

    if self.community_id.is_none() {
      removed_communities = ModRemoveCommunityView::list(&conn, self.mod_user_id, self.page, self.limit)?;
      banned = ModBanView::list(&conn, self.mod_user_id, self.page, self.limit)?;
      added = ModAddView::list(&conn, self.mod_user_id, self.page, self.limit)?;
    }

    // Return the jwt
    Ok(
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
        )?
      )
  }
}

impl Perform for GetReplies {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetReplies
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&self.sort)?;

    let replies = ReplyView::get_replies(&conn, user_id, &sort, self.unread_only, self.page, self.limit)?;

    // Return the jwt
    Ok(
      serde_json::to_string(
        &GetRepliesResponse {
          op: self.op_type().to_string(),
          replies: replies,
        }
        )?
      )
  }
}

impl Perform for BanFromCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::BanFromCommunity
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
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
          return Err(self.error("Community user ban already exists"))?
        }
      };
    } else {
      match CommunityUserBan::unban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(self.error("Community user ban already exists"))?
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
    ModBanFromCommunity::create(&conn, &form)?;

    let user_view = UserView::read(&conn, self.user_id)?;

    let res = serde_json::to_string(
      &BanFromCommunityResponse {
        op: self.op_type().to_string(), 
        user: user_view,
        banned: self.ban
      }
      )
      ?;


    chat.send_community_message(&conn, self.community_id, &res, addr)?;

    Ok(res)
  }
}

impl Perform for AddModToCommunity {
  fn op_type(&self) -> UserOperation {
    UserOperation::AddModToCommunity
  }

  fn perform(&self, chat: &mut ChatServer, addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
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
          return Err(self.error("Community moderator already exists."))?
        }
      };
    } else {
      match CommunityModerator::leave(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(self.error("Community moderator already exists."))?
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
    ModAddCommunity::create(&conn, &form)?;

    let moderators = CommunityModeratorView::for_community(&conn, self.community_id)?;

    let res = serde_json::to_string(
      &AddModToCommunityResponse {
        op: self.op_type().to_string(), 
        moderators: moderators,
      }
      )
      ?;


    chat.send_community_message(&conn, self.community_id, &res, addr)?;

    Ok(res)

  }
}

impl Perform for CreateSite {
  fn op_type(&self) -> UserOperation {
    UserOperation::CreateSite
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    if has_slurs(&self.name) || 
      (self.description.is_some() && has_slurs(&self.description.to_owned().unwrap())) {
        return Err(self.error("No slurs"))?
      }

    let user_id = claims.id;

    // Make sure user is an admin
    if !UserView::read(&conn, user_id)?.admin {
      return Err(self.error("Not an admin."))?
    }

    let site_form = SiteForm {
      name: self.name.to_owned(),
      description: self.description.to_owned(),
      creator_id: user_id,
      updated: None,
    };

    match Site::create(&conn, &site_form) {
      Ok(site) => site,
      Err(_e) => {
        return Err(self.error("Site exists already"))?
      }
    };

    let site_view = SiteView::read(&conn)?;

    Ok(
      serde_json::to_string(
        &SiteResponse {
          op: self.op_type().to_string(), 
          site: site_view,
        }
        )?
      )
  }
}

impl Perform for EditSite {
  fn op_type(&self) -> UserOperation {
    UserOperation::EditSite
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    if has_slurs(&self.name) || 
      (self.description.is_some() && has_slurs(&self.description.to_owned().unwrap())) {
        return Err(self.error("No slurs"))?
      }

    let user_id = claims.id;

    // Make sure user is an admin
    if UserView::read(&conn, user_id)?.admin == false {
      return Err(self.error("Not an admin."))?
    }

    let found_site = Site::read(&conn, 1)?;

    let site_form = SiteForm {
      name: self.name.to_owned(),
      description: self.description.to_owned(),
      creator_id: found_site.creator_id,
      updated: Some(naive_now()),
    };

    match Site::update(&conn, 1, &site_form) {
      Ok(site) => site,
      Err(_e) => {
        return Err(self.error("Couldn't update site."))?
      }
    };

    let site_view = SiteView::read(&conn)?;

    Ok(
      serde_json::to_string(
        &SiteResponse {
          op: self.op_type().to_string(), 
          site: site_view,
        }
        )?
      )
  }
}

impl Perform for GetSite {
  fn op_type(&self) -> UserOperation {
    UserOperation::GetSite
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    // It can return a null site in order to redirect
    let site_view = match Site::read(&conn, 1) {
      Ok(_site) => Some(SiteView::read(&conn)?),
      Err(_e) => None
    };

    let admins = UserView::admins(&conn)?;
    let banned = UserView::banned(&conn)?;

    Ok(
      serde_json::to_string(
        &GetSiteResponse {
          op: self.op_type().to_string(), 
          site: site_view,
          admins: admins,
          banned: banned,
        }
        )?    
      )
  }
}

impl Perform for AddAdmin {
  fn op_type(&self) -> UserOperation {
    UserOperation::AddAdmin
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if UserView::read(&conn, user_id)?.admin == false {
      return Err(self.error("Not an admin."))?
    }

    let read_user = User_::read(&conn, self.user_id)?;

    let user_form = UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(naive_now()),
      admin: self.added,
      banned: read_user.banned,
    };

    match User_::update(&conn, self.user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(self.error("Couldn't update user"))?
      }
    };

    // Mod tables
    let form = ModAddForm {
      mod_user_id: user_id,
      other_user_id: self.user_id,
      removed: Some(!self.added),
    };

    ModAdd::create(&conn, &form)?;

    let admins = UserView::admins(&conn)?;

    let res = serde_json::to_string(
      &AddAdminResponse {
        op: self.op_type().to_string(), 
        admins: admins,
      }
      )
      ?;


    Ok(res)

  }
}

impl Perform for BanUser {
  fn op_type(&self) -> UserOperation {
    UserOperation::BanUser
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let claims = match Claims::decode(&self.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(self.error("Not logged in."))?
      }
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if UserView::read(&conn, user_id)?.admin == false {
      return Err(self.error("Not an admin."))?
    }

    let read_user = User_::read(&conn, self.user_id)?;

    let user_form = UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(naive_now()),
      admin: read_user.admin,
      banned: self.ban,
    };

    match User_::update(&conn, self.user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(self.error("Couldn't update user"))?
      }
    };

    // Mod tables
    let expires = match self.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None
    };

    let form = ModBanForm {
      mod_user_id: user_id,
      other_user_id: self.user_id,
      reason: self.reason.to_owned(),
      banned: Some(self.ban),
      expires: expires,
    };

    ModBan::create(&conn, &form)?;

    let user_view = UserView::read(&conn, self.user_id)?;

    let res = serde_json::to_string(
      &BanUserResponse {
        op: self.op_type().to_string(), 
        user: user_view,
        banned: self.ban
      }
      )
      ?;

    Ok(res)

  }
}

impl Perform for Search {
  fn op_type(&self) -> UserOperation {
    UserOperation::Search
  }

  fn perform(&self, _chat: &mut ChatServer, _addr: usize) -> Result<String, Error> {

    let conn = establish_connection();

    let sort = SortType::from_str(&self.sort)?;
    let type_ = SearchType::from_str(&self.type_)?;

    let mut posts = Vec::new();
    let mut comments = Vec::new();

    match type_ {
      SearchType::Posts => {
        posts = PostView::list(&conn, 
                               PostListingType::All, 
                               &sort, 
                               self.community_id, 
                               None,
                               Some(self.q.to_owned()),
                               None, 
                               false, 
                               false, 
                               self.page, 
                               self.limit)?;
      },
      SearchType::Comments => {
        comments = CommentView::list(&conn, 
                                   &sort, 
                                     None, 
                                     None, 
                                     Some(self.q.to_owned()),
                                     None,
                                     false, 
                                     self.page,
                                     self.limit)?;
      }, 
      SearchType::Both => {
        posts = PostView::list(&conn, 
                               PostListingType::All, 
                               &sort, 
                               self.community_id, 
                               None,
                               Some(self.q.to_owned()),
                               None, 
                               false, 
                               false, 
                               self.page, 
                               self.limit)?;
        comments = CommentView::list(&conn, 
                                   &sort, 
                                     None, 
                                     None, 
                                     Some(self.q.to_owned()),
                                     None,
                                     false, 
                                     self.page,
                                     self.limit)?;
      }
    };


    // Return the jwt
    Ok(
      serde_json::to_string(
        &SearchResponse {
          op: self.op_type().to_string(),
          comments: comments,
          posts: posts,
        }
        )?
      )
  }
}
