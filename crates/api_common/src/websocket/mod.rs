use lemmy_utils::error::LemmyError;
use serde::Serialize;

pub mod chat_server;
pub mod handlers;
pub mod send;
pub mod structs;

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
  SaveComment,
  CreateCommentLike,
  CreateCommentReport,
  ResolveCommentReport,
  ListCommentReports,
  CreatePostLike,
  LockPost,
  FeaturePost,
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
  MarkCommentReplyAsRead,
  GetModlog,
  BanFromCommunity,
  AddModToCommunity,
  AddAdmin,
  GetUnreadRegistrationApplicationCount,
  ListRegistrationApplications,
  ApproveRegistrationApplication,
  BanPerson,
  GetBannedPersons,
  MarkAllAsRead,
  SaveUserSettings,
  TransferCommunity,
  LeaveAdmin,
  PasswordReset,
  PasswordChange,
  MarkPrivateMessageAsRead,
  CreatePrivateMessageReport,
  ResolvePrivateMessageReport,
  ListPrivateMessageReports,
  UserJoin,
  PostJoin,
  CommunityJoin,
  ModJoin,
  ChangePassword,
  GetSiteMetadata,
  BlockCommunity,
  BlockPerson,
  PurgePerson,
  PurgeCommunity,
  PurgePost,
  PurgeComment,
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
  EditCommunity,
  DeleteCommunity,
  RemoveCommunity,
  // Post
  CreatePost,
  GetPost,
  EditPost,
  DeletePost,
  RemovePost,
  // Comment
  CreateComment,
  GetComment,
  EditComment,
  DeleteComment,
  RemoveComment,
  // User
  Register,
  DeleteAccount,
  // Private Message
  CreatePrivateMessage,
  GetPrivateMessages,
  EditPrivateMessage,
  DeletePrivateMessage,
  //Emojis
  CreateCustomEmoji,
  EditCustomEmoji,
  DeleteCustomEmoji,
}

#[derive(EnumString, Display, Debug, Clone)]
pub enum UserOperationApub {
  GetPosts,
  GetCommunity,
  GetComments,
  GetPersonDetails,
  Search,
  ResolveObject,
}

pub trait OperationType {}

impl OperationType for UserOperationCrud {}

impl OperationType for UserOperation {}

impl OperationType for UserOperationApub {}
