#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate anyhow;
extern crate comrak;
extern crate lettre;
extern crate lettre_email;
extern crate openssl;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate thiserror;
extern crate url;

pub mod apub;
pub mod email;
pub mod settings;
#[cfg(test)]
mod test;
pub mod utils;

use crate::settings::Settings;
use regex::Regex;
use thiserror::Error;

pub type ConnectionId = usize;
pub type PostId = i32;
pub type CommunityId = i32;
pub type UserId = i32;
pub type IPAddr = String;

#[macro_export]
macro_rules! location_info {
  () => {
    format!(
      "None value at {}:{}, column {}",
      file!(),
      line!(),
      column!()
    )
  };
}

#[derive(Debug, Error)]
#[error("{{\"error\":\"{message}\"}}")]
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

#[derive(Debug)]
pub struct LemmyError {
  pub inner: anyhow::Error,
}

impl<T> From<T> for LemmyError
where
  T: Into<anyhow::Error>,
{
  fn from(t: T) -> Self {
    LemmyError { inner: t.into() }
  }
}

impl std::fmt::Display for LemmyError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.inner.fmt(f)
  }
}

impl actix_web::error::ResponseError for LemmyError {}

lazy_static! {
  pub static ref WEBFINGER_COMMUNITY_REGEX: Regex = Regex::new(&format!(
    "^group:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
  pub static ref WEBFINGER_USER_REGEX: Regex = Regex::new(&format!(
    "^acct:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
  pub static ref CACHE_CONTROL_REGEX: Regex =
    Regex::new("^((text|image)/.+|application/javascript)$").unwrap();
}
