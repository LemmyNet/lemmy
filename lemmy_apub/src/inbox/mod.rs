use activitystreams::base::{BaseExt, Extends};
use anyhow::Context;
use lemmy_db::{activity::Activity, DbPool};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use serde::{export::fmt::Debug, Serialize};
use url::Url;

pub mod community_inbox;
pub mod shared_inbox;
pub mod user_inbox;

pub(crate) fn get_activity_id<T, Kind>(activity: &T, creator_uri: &Url) -> Result<Url, LemmyError>
where
  T: BaseExt<Kind> + Extends<Kind> + Debug,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let creator_domain = creator_uri.host_str().context(location_info!())?;
  let activity_id = activity.id(creator_domain)?;
  Ok(activity_id.context(location_info!())?.to_owned())
}

pub(crate) async fn is_activity_already_known(
  pool: &DbPool,
  activity_id: &Url,
) -> Result<bool, LemmyError> {
  let activity_id = activity_id.to_string();
  let existing = blocking(pool, move |conn| {
    Activity::read_from_apub_id(&conn, &activity_id)
  })
  .await?;
  match existing {
    Ok(_) => Ok(true),
    Err(_) => Ok(false),
  }
}
