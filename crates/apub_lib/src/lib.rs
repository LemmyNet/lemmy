pub mod activity_queue;
pub mod data;
pub mod object_id;
pub mod signatures;
pub mod traits;
pub mod utils;
pub mod values;
pub mod verify;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Domain did not pass verification")]
  DomainError,
  #[error("Object was not found in database")]
  NotFound,
  #[error("Request limit was reached during fetch")]
  RequestLimit,
  #[error("Object to be fetched was deleted")]
  ObjectDeleted,
  #[error("Private key is missing or invalid")]
  PrivateKeyError,
  #[error("Error during creation of HTTP signature")]
  SigningError(),
  #[error("HTTP signature could not be verified")]
  InvalidSignatureError(),
  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl Error {
  pub fn conv<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    Error::Other(error.into())
  }
}
