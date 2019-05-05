use serde::{Deserialize, Serialize};
use failure::Error;
use db::*;
use db::community::*;
use db::user::*;
use db::post::*;
use db::comment::*;
use db::post_view::*;
use db::comment_view::*;
use db::category::*;
use db::community_view::*;
use db::user_view::*;
use db::moderator_views::*;
use db::moderator::*;
use {has_slurs, remove_slurs, Settings, naive_now, naive_from_unix};

pub mod user;
pub mod community;
pub mod post;
pub mod comment;
pub mod site;

#[derive(EnumString,ToString,Debug)]
pub enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, SaveComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, SavePost, EditCommunity, FollowCommunity, GetFollowedCommunities, GetUserDetails, GetReplies, GetModlog, BanFromCommunity, AddModToCommunity, CreateSite, EditSite, GetSite, AddAdmin, BanUser, Search, MarkAllAsRead
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
