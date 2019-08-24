use serde::{Deserialize, Serialize};
use failure::Error;
use crate::db::*;
use crate::db::community::*;
use crate::db::user::*;
use crate::db::post::*;
use crate::db::comment::*;
use crate::db::post_view::*;
use crate::db::comment_view::*;
use crate::db::category::*;
use crate::db::community_view::*;
use crate::db::user_view::*;
use crate::db::moderator_views::*;
use crate::db::moderator::*;
use crate::{has_slurs, remove_slurs, Settings, naive_now, naive_from_unix};

pub mod user;
pub mod community;
pub mod post;
pub mod comment;
pub mod site;

#[derive(EnumString,ToString,Debug)]
pub enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, SaveComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, SavePost, EditCommunity, FollowCommunity, GetFollowedCommunities, GetUserDetails, GetReplies, GetModlog, BanFromCommunity, AddModToCommunity, CreateSite, EditSite, GetSite, AddAdmin, BanUser, Search, MarkAllAsRead, SaveUserSettings, TransferCommunity, TransferSite
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
  data: T
}

impl <T> Oper<T> {
  pub fn new(op: UserOperation, data: T) -> Oper<T> {
    Oper {
      op: op,
      data: data
    }
  }
}

pub trait Perform<T> {
  fn perform(&self) -> Result<T, Error> where T: Sized;
}
