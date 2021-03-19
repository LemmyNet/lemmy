#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate strum_macros;

pub mod apub;
pub mod claims;
pub mod email;
pub mod rate_limit;
pub mod request;
pub mod settings;

#[cfg(test)]
mod test;
pub mod utils;
pub mod version;

use crate::settings::structs::Settings;
use http::StatusCode;
use regex::Regex;
use std::fmt;
use thiserror::Error;

pub type ConnectionId = usize;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct IpAddr(pub String);

impl fmt::Display for IpAddr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

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
pub struct ApiError {
  pub message: String,
}

impl ApiError {
  pub fn err(msg: &str) -> Self {
    ApiError {
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

impl actix_web::error::ResponseError for LemmyError {
  fn status_code(&self) -> StatusCode {
    match self.inner.downcast_ref::<diesel::result::Error>() {
      Some(diesel::result::Error::NotFound) => StatusCode::NOT_FOUND,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

lazy_static! {
  pub static ref WEBFINGER_COMMUNITY_REGEX: Regex = Regex::new(&format!(
    "^group:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname()
  ))
  .expect("compile webfinger regex");
  pub static ref WEBFINGER_USERNAME_REGEX: Regex = Regex::new(&format!(
    "^acct:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname()
  ))
  .expect("compile webfinger regex");
}
