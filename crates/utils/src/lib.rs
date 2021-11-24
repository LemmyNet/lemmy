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

use actix_web::HttpResponse;
use http::StatusCode;
use std::{fmt, fmt::Display};
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

#[derive(serde::Serialize)]
struct ApiError {
  error: String,
}

pub struct LemmyError {
  pub message: Option<String>,
  pub inner: anyhow::Error,
  pub context: SpanTrace,
}

impl LemmyError {
  pub fn from_message(message: String) -> Self {
    let inner = anyhow::anyhow!("{}", message);
    LemmyError {
      message: Some(message),
      inner,
      context: SpanTrace::capture(),
    }
  }
  pub fn with_message(self, message: String) -> Self {
    LemmyError {
      message: Some(message),
      ..self
    }
  }
}

impl<T> From<T> for LemmyError
where
  T: Into<anyhow::Error>,
{
  fn from(t: T) -> Self {
    LemmyError {
      message: None,
      inner: t.into(),
      context: SpanTrace::capture(),
    }
  }
}

impl std::fmt::Debug for LemmyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("LemmyError")
      .field("message", &self.message)
      .field("inner", &self.inner)
      .field("context", &"SpanTrace")
      .finish()
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
      _ => StatusCode::BAD_REQUEST,
    }
  }

  fn error_response(&self) -> HttpResponse {
    if let Some(message) = &self.message {
      HttpResponse::build(self.status_code()).json(ApiError {
        error: message.clone(),
      })
    } else {
      HttpResponse::build(self.status_code())
        .content_type("text/plain")
        .body(self.inner.to_string())
    }
  }
}
