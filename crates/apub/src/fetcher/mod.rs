pub mod community;
mod fetch;
pub mod objects;
pub mod person;
pub mod search;

use crate::{
  fetcher::{
    community::get_or_fetch_and_upsert_community,
    fetch::FetchError,
    person::get_or_fetch_and_upsert_person,
  },
  ActorType,
};
use chrono::NaiveDateTime;
use http::StatusCode;
use lemmy_db_schema::naive_now;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

static ACTOR_REFETCH_INTERVAL_SECONDS: i64 = 24 * 60 * 60;
static ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG: i64 = 10;

fn is_deleted<Response>(fetch_response: &Result<Response, FetchError>) -> bool
where
  Response: for<'de> Deserialize<'de>,
{
  if let Err(e) = fetch_response {
    if let Some(status) = e.status_code {
      if status == StatusCode::GONE {
        return true;
      }
    }
  }
  false
}

/// Get a remote actor from its apub ID (either a person or a community). Thin wrapper around
/// `get_or_fetch_and_upsert_person()` and `get_or_fetch_and_upsert_community()`.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_actor(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Box<dyn ActorType>, LemmyError> {
  let community = get_or_fetch_and_upsert_community(apub_id, context, recursion_counter).await;
  let actor: Box<dyn ActorType> = match community {
    Ok(c) => Box::new(c),
    Err(_) => Box::new(get_or_fetch_and_upsert_person(apub_id, context, recursion_counter).await?),
  };
  Ok(actor)
}

/// Determines when a remote actor should be refetched from its instance. In release builds, this is
/// `ACTOR_REFETCH_INTERVAL_SECONDS` after the last refetch, in debug builds
/// `ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG`.
///
/// TODO it won't pick up new avatars, summaries etc until a day after.
/// Actors need an "update" activity pushed to other servers to fix this.
fn should_refetch_actor(last_refreshed: NaiveDateTime) -> bool {
  let update_interval = if cfg!(debug_assertions) {
    // avoid infinite loop when fetching community outbox
    chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG)
  } else {
    chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS)
  };
  last_refreshed.lt(&(naive_now() - update_interval))
}
