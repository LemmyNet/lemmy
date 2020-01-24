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
use crate::db::private_message::*;
use crate::db::private_message_view::*;
use crate::db::site::*;
use crate::db::site_view::*;
use crate::db::user::*;
use crate::db::user_mention::*;
use crate::db::user_mention_view::*;
use crate::db::user_view::*;
use crate::db::*;
use crate::{extract_usernames, has_slurs, naive_from_unix, naive_now, remove_slurs};
use diesel::PgConnection;
use failure::Error;
use serde::{Deserialize, Serialize};

pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;

#[derive(Fail, Debug)]
#[fail(display = "{{\"error\":\"{}\"}}", message)]
pub struct APIError {
  pub message: String,
}

impl APIError {
  pub fn err(msg: &str) -> Self {
    APIError {
      message: msg.to_string(),
    }
  }
}

pub struct Oper<T> {
  data: T,
}

impl<T> Oper<T> {
  pub fn new(data: T) -> Oper<T> {
    Oper { data }
  }
}

pub trait Perform<T> {
  fn perform(&self, conn: &PgConnection) -> Result<T, Error>
  where
    T: Sized;
}
