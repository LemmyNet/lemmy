use crate::{data::Data, signatures::PublicKey};
use chrono::NaiveDateTime;
pub use lemmy_apub_lib_derive::*;
use lemmy_utils::LemmyError;
use url::Url;

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

#[async_trait::async_trait(?Send)]
pub trait ApubObject {
  type DataType;
  type ApubType;
  type TombstoneType;

  /// If this object should be refetched after a certain interval, it should return the last refresh
  /// time here. This is mainly used to update remote actors.
  fn last_refreshed_at(&self) -> Option<NaiveDateTime>;
  /// Try to read the object with given ID from local database. Returns Ok(None) if it doesn't exist.
  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError>
  where
    Self: Sized;
  /// Marks the object as deleted in local db. Called when a tombstone is received.
  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError>;

  /// Trait for converting an object or actor into the respective ActivityPub type.
  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError>;
  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError>;

  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError>;

  /// Converts an object from ActivityPub type to Lemmy internal type.
  ///
  /// * `apub` The object to read from
  /// * `context` LemmyContext which holds DB pool, HTTP client etc
  /// * `expected_domain` Domain where the object was received from. None in case of mod action.
  /// * `mod_action_allowed` True if the object can be a mod activity, ignore `expected_domain` in this case
  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

/// Common methods provided by ActivityPub actors (community and person). Not all methods are
/// implemented by all actors.
pub trait ActorType {
  fn actor_id(&self) -> Url;

  fn public_key(&self) -> String;
  fn private_key(&self) -> Option<String>;

  fn inbox_url(&self) -> Url;

  fn shared_inbox_url(&self) -> Option<Url>;

  fn shared_inbox_or_inbox_url(&self) -> Url {
    self.shared_inbox_url().unwrap_or_else(|| self.inbox_url())
  }

  fn get_public_key(&self) -> Result<PublicKey, LemmyError> {
    Ok(PublicKey {
      id: format!("{}#main-key", self.actor_id()),
      owner: Box::new(self.actor_id()),
      public_key_pem: self.public_key(),
    })
  }
}
