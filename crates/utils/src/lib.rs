#[macro_use]
extern crate lazy_static;
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
use log::warn;
use std::{fmt, fmt::Display};
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
}

impl From<deadpool_sync::InteractError> for LemmyError {
  fn from(e: deadpool_sync::InteractError) -> Self {
    LemmyError {
      inner: anyhow::anyhow!(e.to_string()),
    }
  }
}

impl From<deadpool::managed::PoolError<deadpool_diesel::Error>> for LemmyError {
  fn from(e: deadpool::managed::PoolError<deadpool_diesel::Error>) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<deadpool::managed::BuildError<deadpool_diesel::Error>> for LemmyError {
  fn from(e: deadpool::managed::BuildError<deadpool_diesel::Error>) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<jsonwebtoken::errors::Error> for LemmyError {
  fn from(e: jsonwebtoken::errors::Error) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<std::io::Error> for LemmyError {
  fn from(e: std::io::Error) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<deser_hjson::Error> for LemmyError {
  fn from(e: deser_hjson::Error) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<actix_web::error::PayloadError> for LemmyError {
  fn from(e: actix_web::error::PayloadError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<std::time::SystemTimeError> for LemmyError {
  fn from(e: std::time::SystemTimeError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<ApiError> for LemmyError {
  fn from(e: ApiError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<reqwest::Error> for LemmyError {
  fn from(e: reqwest::Error) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<actix::MailboxError> for LemmyError {
  fn from(e: actix::MailboxError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<actix_web::error::Error> for LemmyError {
  fn from(e: actix_web::error::Error) -> Self {
    LemmyError {
      inner: anyhow::anyhow!(e.to_string()),
    }
  }
}

impl From<http::header::ToStrError> for LemmyError {
  fn from(e: http::header::ToStrError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<request::RecvError> for LemmyError {
  fn from(e: request::RecvError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<serde_json::Error> for LemmyError {
  fn from(e: serde_json::Error) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<strum::ParseError> for LemmyError {
  fn from(e: strum::ParseError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<openssl::error::ErrorStack> for LemmyError {
  fn from(e: openssl::error::ErrorStack) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<std::num::ParseIntError> for LemmyError {
  fn from(e: std::num::ParseIntError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<url::ParseError> for LemmyError {
  fn from(e: url::ParseError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<activitystreams::error::DomainError> for LemmyError {
  fn from(e: activitystreams::error::DomainError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<base64::DecodeError> for LemmyError {
  fn from(e: base64::DecodeError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<diesel::result::Error> for LemmyError {
  fn from(e: diesel::result::Error) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<diesel_migrations::RunMigrationsError> for LemmyError {
  fn from(e: diesel_migrations::RunMigrationsError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<http::header::InvalidHeaderValue> for LemmyError {
  fn from(e: http::header::InvalidHeaderValue) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<http::header::InvalidHeaderName> for LemmyError {
  fn from(e: http::header::InvalidHeaderName) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<http_signature_normalization_reqwest::SignError> for LemmyError {
  fn from(e: http_signature_normalization_reqwest::SignError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<http_signature_normalization_actix::PrepareVerifyError> for LemmyError {
  fn from(e: http_signature_normalization_actix::PrepareVerifyError) -> Self {
    LemmyError { inner: e.into() }
  }
}

impl From<anyhow::Error> for LemmyError {
  fn from(e: anyhow::Error) -> Self {
    LemmyError { inner: e }
  }
}

impl Display for LemmyError {
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

// impl From<deadpool_sync::InteractError> for LemmyError {

// }
