pub extern crate serde;
pub extern crate thiserror;

pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;

use thiserror::Error;

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
