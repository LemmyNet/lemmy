use crate::data::Data;
use chrono::NaiveDateTime;
pub use lemmy_apub_lib_derive::*;
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
pub trait ActivityHandler {
  type DataType;
  fn id(&self) -> &Url;
  fn actor(&self) -> &Url;
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
  type DbType;
  type TombstoneType;

  /// If the object is stored in the database, this method should return the fetch time. Used to
  /// update actors after certain interval.
  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }
  /// Try to read the object with given ID from local database. Returns Ok(None) if it doesn't exist.
  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError>
  where
    Self: Sized;
  /// Marks the object as deleted in local db. Called when a delete activity is received, or if
  /// fetch returns a tombstone.
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
