pub mod values;

use activitystreams::error::DomainError;
pub use lemmy_apub_lib_derive::*;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod webfinger;

pub trait ActivityFields {
  fn id_unchecked(&self) -> &Url;
  fn actor(&self) -> &Url;
  fn cc(&self) -> Vec<Url>;
}

#[async_trait::async_trait(?Send)]
pub trait ActivityHandler {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError>;

  async fn receive(
    self,
    context: &LemmyContext,
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
