//! The Lemmy websocket crate

#![deny(missing_docs)]
#[macro_use]
extern crate strum_macros;

use crate::chat_server::ChatServer;
use actix::Addr;
use background_jobs::QueueHandle;
use lemmy_db::DbPool;
use lemmy_utils::LemmyError;
use reqwest::Client;
use serde::Serialize;

/// The chat server
pub mod chat_server;

/// The websocket handlers
pub mod handlers;

/// The websocket messages
pub mod messages;

/// The lemmy websocket context
pub struct LemmyContext {
  /// The DB pool
  pub pool: DbPool,
  /// The chat server
  pub chat_server: Addr<ChatServer>,
  /// The http client
  pub client: Client,
  /// The activity queue
  pub activity_queue: QueueHandle,
}

impl LemmyContext {
  /// Create a lemmy context
  pub fn create(
    pool: DbPool,
    chat_server: Addr<ChatServer>,
    client: Client,
    activity_queue: QueueHandle,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      chat_server,
      client,
      activity_queue,
    }
  }

  /// The DB pool
  pub fn pool(&self) -> &DbPool {
    &self.pool
  }

  /// The chat server
  pub fn chat_server(&self) -> &Addr<ChatServer> {
    &self.chat_server
  }

  /// The http client
  pub fn client(&self) -> &Client {
    &self.client
  }

  /// The activity queue
  pub fn activity_queue(&self) -> &QueueHandle {
    &self.activity_queue
  }
}

impl Clone for LemmyContext {
  fn clone(&self) -> Self {
    LemmyContext {
      pool: self.pool.clone(),
      chat_server: self.chat_server.clone(),
      client: self.client.clone(),
      activity_queue: self.activity_queue.clone(),
    }
  }
}

#[derive(Serialize)]
struct WebsocketResponse<T> {
  op: String,
  data: T,
}

/// Serialize the websocket message
pub fn serialize_websocket_message<Response>(
  op: &UserOperation,
  data: &Response,
) -> Result<String, LemmyError>
where
  Response: Serialize,
{
  let response = WebsocketResponse {
    op: op.to_string(),
    data,
  };
  Ok(serde_json::to_string(&response)?)
}

#[derive(EnumString, ToString, Debug, Clone)]
/// The User operations
pub enum UserOperation {
  /// Login
  Login,
  /// Register,
  Register,
  /// Get the Captcha
  GetCaptcha,
  /// Create a community
  CreateCommunity,
  /// Create a post
  CreatePost,
  /// List communities
  ListCommunities,
  /// List categories
  ListCategories,
  /// Get a post
  GetPost,
  /// Get a community
  GetCommunity,
  /// Create a comment
  CreateComment,
  /// Edit a comment
  EditComment,
  /// Delete a comment
  DeleteComment,
  /// Remove a comment
  RemoveComment,
  /// Mark a comment as read
  MarkCommentAsRead,
  /// Save a comment
  SaveComment,
  /// Create a comment like
  CreateCommentLike,
  /// Get posts
  GetPosts,
  /// Create a post like
  CreatePostLike,
  /// Edit a post
  EditPost,
  /// Delete a post
  DeletePost,
  /// Remove a post
  RemovePost,
  /// Lock a post
  LockPost,
  /// Sticky a post
  StickyPost,
  /// Save a post
  SavePost,
  /// Edit a community
  EditCommunity,
  /// Delete a community
  DeleteCommunity,
  /// Remove a community
  RemoveCommunity,
  /// Follow a community
  FollowCommunity,
  /// Get followed communities
  GetFollowedCommunities,
  /// Get user details
  GetUserDetails,
  /// Get replies
  GetReplies,
  /// Get mentions
  GetUserMentions,
  /// Mark mention as read
  MarkUserMentionAsRead,
  /// Get modlog
  GetModlog,
  /// Ban from community
  BanFromCommunity,
  /// Add mod to community
  AddModToCommunity,
  /// Create site
  CreateSite,
  /// Edit site
  EditSite,
  /// Get site
  GetSite,
  /// Add admin
  AddAdmin,
  /// Ban user
  BanUser,
  /// Search
  Search,
  /// Mark all as read
  MarkAllAsRead,
  /// Save user settings
  SaveUserSettings,
  /// Transfer community
  TransferCommunity,
  /// Transfer site
  TransferSite,
  /// Delete account
  DeleteAccount,
  /// Reset password
  PasswordReset,
  /// Change password
  PasswordChange,
  /// Create private message
  CreatePrivateMessage,
  /// Edit privet message
  EditPrivateMessage,
  /// Delete private message
  DeletePrivateMessage,
  /// Mark private message as read
  MarkPrivateMessageAsRead,
  /// Get private messages
  GetPrivateMessages,
  /// Join a user room
  UserJoin,
  /// Get comments
  GetComments,
  /// Get the site config
  GetSiteConfig,
  /// Save the site config
  SaveSiteConfig,
  /// Join a post room
  PostJoin,
  /// Join a community room
  CommunityJoin,
}
