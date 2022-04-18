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

mod sensitive;

pub use sensitive::Sensitive;

use actix_web::HttpResponse;
use http::StatusCode;
use std::{fmt, fmt::Display, time::Duration};
use tracing_error::SpanTrace;

pub type ConnectionId = usize;

pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

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
  error: &'static str,
}

pub struct LemmyError {
  pub message: Option<&'static str>,
  pub inner: anyhow::Error,
  pub context: SpanTrace,
}

impl LemmyError {
  /// Create LemmyError from a message, including stack trace
  pub fn from_message(message: &'static str) -> Self {
    let inner = anyhow::anyhow!("{}", message);
    LemmyError {
      message: Some(message),
      inner,
      context: SpanTrace::capture(),
    }
  }

  /// Create a LemmyError from error and message, including stack trace
  pub fn from_error_message<E>(error: E, message: &'static str) -> Self
  where
    E: Into<anyhow::Error>,
  {
    LemmyError {
      message: Some(message),
      inner: error.into(),
      context: SpanTrace::capture(),
    }
  }

  /// Add message to existing LemmyError (or overwrite existing error)
  pub fn with_message(self, message: &'static str) -> Self {
    LemmyError {
      message: Some(message),
      ..self
    }
  }

  pub fn to_json(&self) -> Result<String, Self> {
    let api_error = match self.message {
      Some(error) => ApiError { error },
      None => ApiError { error: "Unknown" },
    };

    Ok(serde_json::to_string(&api_error)?)
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
    if let Some(message) = self.message {
      write!(f, "{}: ", message)?;
    }
    writeln!(f, "{}", self.inner)?;
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
      HttpResponse::build(self.status_code()).json(ApiError { error: message })
    } else {
      HttpResponse::build(self.status_code())
        .content_type("text/plain")
        .body(self.inner.to_string())
    }
  }
}
