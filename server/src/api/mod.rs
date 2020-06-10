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
use crate::{
  extract_usernames, fetch_iframely_and_pictrs_data, generate_random_string, naive_from_unix,
  naive_now, remove_slurs, send_email, slur_check, slurs_vec_to_str,
};

use crate::settings::Settings;
use crate::websocket::UserOperation;
use crate::websocket::{
  server::{
    JoinCommunityRoom, JoinPostRoom, JoinUserRoom, SendAllMessage, SendComment,
    SendCommunityRoomMessage, SendPost, SendUserRoomMessage,
  },
  WebsocketInfo,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<Self::Response, Error>;
}
