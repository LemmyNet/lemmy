#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate strum_macros;

pub mod apub;
pub mod email;
pub mod rate_limit;
pub mod request;
pub mod settings;
#[cfg(test)]
mod test;
pub mod utils;

use crate::{request::RecvError, settings::Settings};
use activitystreams::{error::DomainError, mime::FromStrError};
use actix_web::error::BlockingError;
use http::{header::ToStrError, StatusCode};
use regex::Regex;
use std::time::SystemTimeError;
use thiserror::Error;
use openssl::error::ErrorStack;
use http::header::InvalidHeaderName;
use http::header::InvalidHeaderValue;
use http_signature_normalization_actix::PrepareVerifyError;
use http_signature_normalization_actix::PrepareSignError;
use http_signature_normalization_reqwest::SignError;
use base64::DecodeError;
use std::num::ParseIntError;

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
  pub status_code: Option<StatusCode>,
}

impl std::fmt::Display for LemmyError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.inner.fmt(f)
  }
}

impl From<anyhow::Error> for LemmyError {
  fn from(e: anyhow::Error) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<APIError> for LemmyError {
  fn from(e: APIError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<SystemTimeError> for LemmyError {
  fn from(e: SystemTimeError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<reqwest::Error> for LemmyError {
  fn from(e: reqwest::Error) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<RecvError> for LemmyError {
  fn from(e: RecvError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<ToStrError> for LemmyError {
  fn from(e: ToStrError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<r2d2::Error> for LemmyError {
  fn from(e: r2d2::Error) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<BlockingError<LemmyError>> for LemmyError {
  fn from(e: BlockingError<LemmyError>) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<serde_json::Error> for LemmyError {
  fn from(e: serde_json::Error) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<strum::ParseError> for LemmyError {
  fn from(e: strum::ParseError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<url::ParseError> for LemmyError {
  fn from(e: url::ParseError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<FromStrError> for LemmyError {
  fn from(e: FromStrError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<DomainError> for LemmyError {
  fn from(e: DomainError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<ErrorStack> for LemmyError {
  fn from(e: ErrorStack) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<InvalidHeaderName> for LemmyError {
  fn from(e: InvalidHeaderName) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<InvalidHeaderValue> for LemmyError {
  fn from(e: InvalidHeaderValue) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<PrepareVerifyError> for LemmyError {
  fn from(e: PrepareVerifyError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<PrepareSignError> for LemmyError {
  fn from(e: PrepareSignError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<SignError> for LemmyError {
  fn from(e: SignError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<DecodeError> for LemmyError {
  fn from(e: DecodeError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<ParseIntError> for LemmyError {
  fn from(e: ParseIntError) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<std::io::Error> for LemmyError {
  fn from(e: std::io::Error) -> Self {
    LemmyError {
      inner: e.into(),
      status_code: None,
    }
  }
}

impl From<diesel::result::Error> for LemmyError {
  fn from(e: diesel::result::Error) -> Self {
    let status_code = match e {
      diesel::result::Error::NotFound => Some(StatusCode::NOT_FOUND),
      _ => None,
    };
    LemmyError {
      inner: e.into(),
      status_code,
    }
  }
}

impl actix_web::error::ResponseError for LemmyError {
  fn status_code(&self) -> StatusCode {
    self
      .status_code
      .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

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
}
