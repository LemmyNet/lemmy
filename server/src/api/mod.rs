use failure::Error;

pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;

#[derive(EnumString, ToString, Debug)]
pub enum UserOperation {
  Login,
  Register,
  CreateCommunity,
  CreatePost,
  ListCommunities,
  ListCategories,
  GetPost,
  GetCommunity,
  CreateComment,
  EditComment,
  SaveComment,
  CreateCommentLike,
  GetPosts,
  CreatePostLike,
  EditPost,
  SavePost,
  EditCommunity,
  FollowCommunity,
  GetFollowedCommunities,
  GetUserDetails,
  GetReplies,
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
}

#[derive(Fail, Debug)]
#[fail(display = "{{\"op\":\"{}\", \"error\":\"{}\"}}", op, message)]
pub struct APIError {
  pub op: String,
  pub message: String,
}

impl APIError {
  pub fn err(op: &UserOperation, msg: &str) -> Self {
    APIError {
      op: op.to_string(),
      message: msg.to_string(),
    }
  }
}

pub struct Oper<T> {
  op: UserOperation,
  data: T,
}

impl<T> Oper<T> {
  pub fn new(op: UserOperation, data: T) -> Oper<T> {
    Oper { op: op, data: data }
  }
}

pub trait Perform<T> {
  fn perform(&self) -> Result<T, Error>
  where
    T: Sized;
}
