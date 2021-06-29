use crate::inbox::new_inbox_routing::Activity;
use anyhow::{anyhow, Context};
use lemmy_api_common::blocking;
use lemmy_db_queries::{source::activity::Activity_, DbPool};
use lemmy_db_schema::source::activity::Activity as DbActivity;
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use std::fmt::Debug;
use url::Url;

pub mod community_inbox;
pub mod new_inbox_routing;
pub mod person_inbox;
pub mod shared_inbox;

pub(crate) async fn is_activity_already_known(
  pool: &DbPool,
  activity_id: &Url,
) -> Result<bool, LemmyError> {
  let activity_id = activity_id.to_owned().into();
  let existing = blocking(pool, move |conn| {
    DbActivity::read_from_apub_id(&conn, &activity_id)
  })
  .await?;
  match existing {
    Ok(_) => Ok(true),
    Err(_) => Ok(false),
  }
}

pub(in crate::inbox) fn assert_activity_not_local<T: Debug>(
  activity: &Activity<T>,
) -> Result<(), LemmyError> {
  let activity_domain = activity.id_unchecked().domain().context(location_info!())?;

  if activity_domain == Settings::get().hostname() {
    return Err(
      anyhow!(
        "Error: received activity which was sent by local instance: {:?}",
        activity
      )
      .into(),
    );
  }
  Ok(())
}
