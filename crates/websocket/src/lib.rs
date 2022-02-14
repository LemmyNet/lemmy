#[macro_use]
extern crate strum_macros;

use crate::chat_server::ChatServer;
use actix::Addr;
use background_jobs::QueueHandle;
use lemmy_db_schema::{source::secret::Secret, DbPool};
use lemmy_utils::{settings::structs::Settings, LemmyError};
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;

pub mod chat_server;
pub mod handlers;
pub mod messages;
pub mod routes;
pub mod send;

pub struct LemmyContext {
  pool: DbPool,
  chat_server: Addr<ChatServer>,
  client: ClientWithMiddleware,
  activity_queue: QueueHandle,
  settings: Settings,
  secret: Secret,
}

impl LemmyContext {
  pub fn create(
    pool: DbPool,
    chat_server: Addr<ChatServer>,
    client: ClientWithMiddleware,
    activity_queue: QueueHandle,
    settings: Settings,
    secret: Secret,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      chat_server,
      client,
      activity_queue,
      settings,
      secret,
    }
  }
  pub fn pool(&self) -> &DbPool {
    &self.pool
  }
  pub fn chat_server(&self) -> &Addr<ChatServer> {
    &self.chat_server
  }
  pub fn client(&self) -> &ClientWithMiddleware {
    &self.client
  }
  pub fn activity_queue(&self) -> &QueueHandle {
    &self.activity_queue
  }
  pub fn settings(&self) -> Settings {
    // TODO hacky solution to be able to hotload the settings.
    Settings::get()
  }
  pub fn secret(&self) -> &Secret {
    &self.secret
  }
}

impl Clone for LemmyContext {
  fn clone(&self) -> Self {
    LemmyContext {
      pool: self.pool.clone(),
      chat_server: self.chat_server.clone(),
      client: self.client.clone(),
      activity_queue: self.activity_queue.clone(),
      settings: self.settings.clone(),
      secret: self.secret.clone(),
    }
  }
}

#[derive(Serialize)]
struct WebsocketResponse<T> {
  op: String,
  data: T,
}

pub fn serialize_websocket_message<OP, Response>(
  op: &OP,
  data: &Response,
) -> Result<String, LemmyError>
where
  Response: Serialize,
  OP: ToString,
{
  let response = WebsocketResponse {
    op: op.to_string(),
    data,
  };
  Ok(serde_json::to_string(&response)?)
}

#[derive(EnumString, Display, Debug, Clone)]
pub enum UserOperation {
  Login,
  GetCaptcha,
  MarkCommentAsRead,
  SaveComment,
  CreateCommentLike,
  CreateCommentReport,
  ResolveCommentReport,
  ListCommentReports,
  CreatePostLike,
  LockPost,
  StickyPost,
  MarkPostAsRead,
  SavePost,
  CreatePostReport,
  ResolvePostReport,
  ListPostReports,
  GetReportCount,
  GetUnreadCount,
  VerifyEmail,
  FollowCommunity,
  GetReplies,
  GetPersonMentions,
  MarkPersonMentionAsRead,
  GetModlog,
  BanFromCommunity,
  AddModToCommunity,
  AddAdmin,
  GetUnreadRegistrationApplicationCount,
  ListRegistrationApplications,
  ApproveRegistrationApplication,
  BanPerson,
  GetBannedPersons,
  Search,
  ResolveObject,
  MarkAllAsRead,
  SaveUserSettings,
  TransferCommunity,
  LeaveAdmin,
  PasswordReset,
  PasswordChange,
  MarkPrivateMessageAsRead,
  UserJoin,
  GetSiteConfig,
  SaveSiteConfig,
  PostJoin,
  CommunityJoin,
  ModJoin,
  ChangePassword,
  GetSiteMetadata,
  BlockCommunity,
  BlockPerson,
}

#[derive(EnumString, Display, Debug, Clone)]
pub enum UserOperationCrud {
  // Site
  CreateSite,
  GetSite,
  EditSite,
  // Community
  CreateCommunity,
  ListCommunities,
  GetCommunity,
  EditCommunity,
  DeleteCommunity,
  RemoveCommunity,
  // Post
  CreatePost,
  GetPost,
  GetPosts,
  EditPost,
  DeletePost,
  RemovePost,
  // Comment
  CreateComment,
  GetComment,
  GetComments,
  EditComment,
  DeleteComment,
  RemoveComment,
  // User
  Register,
  GetPersonDetails,
  DeleteAccount,
  // Private Message
  CreatePrivateMessage,
  GetPrivateMessages,
  EditPrivateMessage,
  DeletePrivateMessage,
}

pub trait OperationType {}

impl OperationType for UserOperationCrud {}

impl OperationType for UserOperation {}
