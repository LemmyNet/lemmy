use actix_web::ResponseError;
use std::fmt::{Display, Formatter};

/// Necessary because of this issue: https://github.com/actix/actix-web/issues/1711
#[derive(Debug)]
pub struct Error(anyhow::Error);

impl ResponseError for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&self.0, f)
  }
}

impl<T> From<T> for Error
where
  T: Into<anyhow::Error>,
{
  fn from(t: T) -> Self {
    Error(t.into())
  }
}
