use crate::{data::Data, signatures::PublicKey};
use activitystreams::chrono::NaiveDateTime;
use anyhow::Context;
pub use lemmy_apub_lib_derive::*;
use lemmy_utils::{location_info, LemmyError};
use url::Url;

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

pub trait ApubObject {
  type DataType;
  /// If this object should be refetched after a certain interval, it should return the last refresh
  /// time here. This is mainly used to update remote actors.
  fn last_refreshed_at(&self) -> Option<NaiveDateTime>;
  /// Try to read the object with given ID from local database. Returns Ok(None) if it doesn't exist.
  fn read_from_apub_id(data: &Self::DataType, object_id: Url) -> Result<Option<Self>, LemmyError>
  where
    Self: Sized;
  /// Marks the object as deleted in local db. Called when a tombstone is received.
  fn delete(self, data: &Self::DataType) -> Result<(), LemmyError>;
}

/// Common methods provided by ActivityPub actors (community and person). Not all methods are
/// implemented by all actors.
pub trait ActorType {
  fn is_local(&self) -> bool;
  fn actor_id(&self) -> Url;
  fn name(&self) -> String;

  // TODO: this should not be an option (needs db migration in lemmy)
  fn public_key(&self) -> Option<String>;
  fn private_key(&self) -> Option<String>;

  fn inbox_url(&self) -> Url;

  fn shared_inbox_url(&self) -> Option<Url>;

  fn shared_inbox_or_inbox_url(&self) -> Url {
    self.shared_inbox_url().unwrap_or_else(|| self.inbox_url())
  }

  fn get_public_key(&self) -> Result<PublicKey, LemmyError> {
    Ok(PublicKey {
      id: format!("{}#main-key", self.actor_id()),
      owner: self.actor_id(),
      public_key_pem: self.public_key().context(location_info!())?,
    })
  }
}
