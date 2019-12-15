use crate::db::category::*;
use crate::db::comment::*;
use crate::db::comment_view::*;
use crate::db::community::*;
use crate::db::community_view::*;
use crate::db::moderator::*;
use crate::db::moderator_views::*;
use crate::db::password_reset_request::*;
use crate::db::post::*;
use crate::db::post_view::*;
use crate::db::site::*;
use crate::db::site_view::*;
use crate::db::user::*;
use crate::db::user_mention::*;
use crate::db::user_mention_view::*;
use crate::db::user_view::*;
use crate::db::*;
use crate::{extract_usernames, has_slurs, naive_from_unix, naive_now, remove_slurs};
use failure::Error;
use serde::{Deserialize, Serialize};

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
  GetUserMentions,
  EditUserMention,
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
    Oper { op, data }
  }
}

pub trait Perform<T> {
  fn perform(&self) -> Result<T, Error>
  where
    T: Sized;
}
