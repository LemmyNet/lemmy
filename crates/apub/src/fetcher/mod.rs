pub mod object_id;
pub mod post_or_comment;
pub mod search;
pub mod user_or_community;

use chrono::NaiveDateTime;
use lemmy_db_schema::naive_now;

static ACTOR_REFETCH_INTERVAL_SECONDS: i64 = 24 * 60 * 60;
static ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG: i64 = 10;

/// Determines when a remote actor should be refetched from its instance. In release builds, this is
/// `ACTOR_REFETCH_INTERVAL_SECONDS` after the last refetch, in debug builds
/// `ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG`.
///
/// TODO it won't pick up new avatars, summaries etc until a day after.
/// Actors need an "update" activity pushed to other servers to fix this.
fn should_refetch_object(last_refreshed: NaiveDateTime) -> bool {
  let update_interval = if cfg!(debug_assertions) {
    // avoid infinite loop when fetching community outbox
    chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG)
  } else {
    chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS)
  };
  last_refreshed.lt(&(naive_now() - update_interval))
}
