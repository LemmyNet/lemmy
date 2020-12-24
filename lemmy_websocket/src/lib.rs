#[macro_use]
extern crate strum_macros;

use crate::chat_server::ChatServer;
use actix::Addr;
use background_jobs::QueueHandle;
use lemmy_db_queries::DbPool;
use lemmy_utils::LemmyError;
use reqwest::Client;
use serde::Serialize;

pub mod chat_server;
pub mod handlers;
pub mod messages;

pub struct LemmyContext {
  pub pool: DbPool,
  pub chat_server: Addr<ChatServer>,
  pub client: Client,
  pub activity_queue: QueueHandle,
}

impl LemmyContext {
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
  pub fn pool(&self) -> &DbPool {
    &self.pool
  }
  pub fn chat_server(&self) -> &Addr<ChatServer> {
    &self.chat_server
  }
  pub fn client(&self) -> &Client {
    &self.client
  }
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
pub enum UserOperation {
  Login,
  Register,
  GetCaptcha,
  CreateCommunity,
  CreatePost,
  ListCommunities,
  ListCategories,
  GetPost,
  GetCommunity,
  CreateComment,
  EditComment,
  DeleteComment,
  RemoveComment,
  MarkCommentAsRead,
  SaveComment,
  CreateCommentLike,
  CreateCommentReport,
  ResolveCommentReport,
  ListCommentReports,
  GetPosts,
  CreatePostLike,
  EditPost,
  DeletePost,
  RemovePost,
  LockPost,
  StickyPost,
  SavePost,
  CreatePostReport,
  ResolvePostReport,
  ListPostReports,
  GetReportCount,
  EditCommunity,
  DeleteCommunity,
  RemoveCommunity,
  FollowCommunity,
  GetFollowedCommunities,
  GetUserDetails,
  GetReplies,
  GetUserMentions,
  MarkUserMentionAsRead,
  GetModlog,
  BanFromCommunity,
  AddModToCommunity,
  CreateSite,
  EditSite,
  GetSite,
  AddAdmin,
  BanUser,
  Search,
  MarkAllAsRead,
  SaveUserSettings,
  TransferCommunity,
  TransferSite,
  DeleteAccount,
  PasswordReset,
  PasswordChange,
  CreatePrivateMessage,
  EditPrivateMessage,
  DeletePrivateMessage,
  MarkPrivateMessageAsRead,
  GetPrivateMessages,
  UserJoin,
  GetComments,
  GetSiteConfig,
  SaveSiteConfig,
  PostJoin,
  CommunityJoin,
  ModJoin,
}
