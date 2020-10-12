use activitystreams::{
  base::BaseExt,
  object::{Tombstone, TombstoneExt},
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use lemmy_utils::{utils::convert_datetime, LemmyError};

pub mod comment;
pub mod community;
pub mod post;
pub mod private_message;
pub mod user;

/// Updated is actually the deletion time
fn create_tombstone<T>(
  deleted: bool,
  object_id: &str,
  updated: Option<NaiveDateTime>,
  former_type: T,
) -> Result<Tombstone, LemmyError>
where
  T: ToString,
{
  if deleted {
    if let Some(updated) = updated {
      let mut tombstone = Tombstone::new();
      tombstone.set_id(object_id.parse()?);
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
