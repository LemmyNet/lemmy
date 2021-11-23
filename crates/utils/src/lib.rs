#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate smart_default;

pub mod apub;
pub mod email;
pub mod rate_limit;
pub mod request;
pub mod settings;

pub mod claims;
#[cfg(test)]
mod test;
pub mod utils;
pub mod version;

use http::StatusCode;
use std::{fmt, fmt::Display};
use thiserror::Error;
use tracing::warn;
use tracing_error::SpanTrace;

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
  message: String,
}

impl ApiError {
  pub fn err_plain(msg: &str) -> Self {
    ApiError {
      message: msg.to_string(),
    }
  }
  pub fn err<E: Display>(msg: &str, original_error: E) -> Self {
    warn!("{}", original_error);
    ApiError {
      message: msg.to_string(),
    }
  }
}

#[derive(Debug)]
pub struct LemmyError {
  pub inner: anyhow::Error,
  pub context: SpanTrace,
}

impl<T> From<T> for LemmyError
where
  T: Into<anyhow::Error>,
{
  fn from(t: T) -> Self {
    LemmyError {
      inner: t.into(),
      context: SpanTrace::capture(),
    }
  }
}

impl Display for LemmyError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.inner.fmt(f)?;
    self.context.fmt(f)
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
