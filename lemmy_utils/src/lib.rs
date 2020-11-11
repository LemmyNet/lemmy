//! The Lemmy utils crate

#![deny(missing_docs)]
#[macro_use]
extern crate lazy_static;

/// Apub utils
pub mod apub;

/// Email utils
pub mod email;

/// Request utils
pub mod request;

/// Settings utils
pub mod settings;
#[cfg(test)]
mod test;

/// General utils
pub mod utils;

use crate::settings::Settings;
use regex::Regex;
use thiserror::Error;

/// The connection id
pub type ConnectionId = usize;
/// The post id
pub type PostId = i32;
/// The community id
pub type CommunityId = i32;
/// The user id
pub type UserId = i32;
/// The IPAddr
pub type IPAddr = String;

#[macro_export]
/// A macro that adds logging info to a None value
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
/// The API error
pub struct APIError {
  /// The API error message
  pub message: String,
}

impl APIError {
  /// Creating an API error
  pub fn err(msg: &str) -> Self {
    APIError {
      message: msg.to_string(),
    }
  }
}

#[derive(Debug)]
/// A Lemmy error type
pub struct LemmyError {
  /// The inner error
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
  /// The webfinger community regex
  pub static ref WEBFINGER_COMMUNITY_REGEX: Regex = Regex::new(&format!(
    "^group:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
  /// The webfinger user regex
  pub static ref WEBFINGER_USER_REGEX: Regex = Regex::new(&format!(
    "^acct:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
}
