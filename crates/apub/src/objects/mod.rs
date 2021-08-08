use crate::fetcher::person::get_or_fetch_and_upsert_person;
use activitystreams::{
  base::BaseExt,
  object::{kind::ImageType, Tombstone, TombstoneExt},
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use lemmy_apub_lib::values::MediaTypeMarkdown;
use lemmy_db_queries::DbPool;
use lemmy_utils::{utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

pub(crate) mod comment;
pub(crate) mod community;
pub(crate) mod person;
pub(crate) mod post;
pub(crate) mod private_message;

/// Trait for converting an object or actor into the respective ActivityPub type.
#[async_trait::async_trait(?Send)]
pub(crate) trait ToApub {
  type ApubType;
  async fn to_apub(&self, pool: &DbPool) -> Result<Self::ApubType, LemmyError>;
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError>;
}

#[async_trait::async_trait(?Send)]
pub(crate) trait FromApub {
  type ApubType;
  /// Converts an object from ActivityPub type to Lemmy internal type.
  ///
  /// * `apub` The object to read from
  /// * `context` LemmyContext which holds DB pool, HTTP client etc
  /// * `expected_domain` Domain where the object was received from. None in case of mod action.
  /// * `mod_action_allowed` True if the object can be a mod activity, ignore `expected_domain` in this case
  async fn from_apub(
    apub: &Self::ApubType,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  content: String,
  media_type: MediaTypeMarkdown,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
  #[serde(rename = "type")]
  kind: ImageType,
  url: Url,
}

/// Updated is actually the deletion time
fn create_tombstone<T>(
  deleted: bool,
  object_id: Url,
  updated: Option<NaiveDateTime>,
  former_type: T,
) -> Result<Tombstone, LemmyError>
where
  T: ToString,
{
  if deleted {
    if let Some(updated) = updated {
      let mut tombstone = Tombstone::new();
      tombstone.set_id(object_id);
      tombstone.set_former_type(former_type.to_string());
      tombstone.set_deleted(convert_datetime(updated));
      Ok(tombstone)
    } else {
      Err(anyhow!("Cant convert to tombstone because updated time was None.").into())
    }
  } else {
    Err(anyhow!("Cant convert object to tombstone if it wasnt deleted").into())
  }
}
