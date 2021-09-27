pub mod values;

use activitystreams::error::DomainError;
pub use lemmy_apub_lib_derive::*;
use lemmy_utils::LemmyError;
use std::{ops::Deref, sync::Arc};
use url::Url;

pub mod webfinger;

pub trait ActivityFields {
  fn id_unchecked(&self) -> &Url;
  fn actor(&self) -> &Url;
  fn cc(&self) -> Vec<Url>;
}

#[async_trait::async_trait(?Send)]
pub trait ActivityHandler {
  type DataType;
  async fn verify(
    &self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError>;

  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError>;
}

pub fn verify_domains_match(a: &Url, b: &Url) -> Result<(), LemmyError> {
  if a.domain() != b.domain() {
    return Err(DomainError.into());
  }
  Ok(())
}

pub fn verify_urls_match(a: &Url, b: &Url) -> Result<(), LemmyError> {
  if a != b {
    return Err(DomainError.into());
  }
  Ok(())
}

#[derive(Debug)]
pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
  /// Create new `Data` instance.
  pub fn new(state: T) -> Data<T> {
    Data(Arc::new(state))
  }

  /// Get reference to inner app data.
  pub fn get_ref(&self) -> &T {
    self.0.as_ref()
  }

  /// Convert to the internal Arc<T>
  pub fn into_inner(self) -> Arc<T> {
    self.0
  }
}

impl<T: ?Sized> Deref for Data<T> {
  type Target = Arc<T>;

  fn deref(&self) -> &Arc<T> {
    &self.0
  }
}

impl<T: ?Sized> Clone for Data<T> {
  fn clone(&self) -> Data<T> {
    Data(self.0.clone())
  }
}
