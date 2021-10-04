use crate::data::Data;
use activitystreams::chrono::NaiveDateTime;
pub use lemmy_apub_lib_derive::*;
use lemmy_utils::LemmyError;
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
}
