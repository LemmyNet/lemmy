use crate::check_is_apub_id_valid;
use activitystreams::{
  base::{AsBase, BaseExt},
  markers::Base,
  object::{Tombstone, TombstoneExt},
};
use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use lemmy_utils::{location_info, utils::convert_datetime, LemmyError};
use url::Url;

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

pub(in crate::objects) fn check_object_domain<T, Kind>(
  apub: &T,
  expected_domain: Option<Url>,
) -> Result<String, LemmyError>
where
  T: Base + AsBase<Kind>,
{
  let actor_id = if let Some(url) = expected_domain {
    check_is_apub_id_valid(&url)?;
    let domain = url.domain().context(location_info!())?;
    apub.id(domain)?.context(location_info!())?
  } else {
    let actor_id = apub.id_unchecked().context(location_info!())?;
    check_is_apub_id_valid(&actor_id)?;
    actor_id
  };
  Ok(actor_id.to_string())
}
