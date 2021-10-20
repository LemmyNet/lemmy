use activitystreams::{
  base::BaseExt,
  object::{kind::ImageType, Tombstone, TombstoneExt},
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use lemmy_apub_lib::values::MediaTypeMarkdown;
use lemmy_utils::{utils::convert_datetime, LemmyError};
use url::Url;

pub mod comment;
pub mod community;
pub mod person;
pub mod post;
pub mod private_message;

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
