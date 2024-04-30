#[cfg(feature = "full")]
pub mod build_response;
#[cfg(feature = "full")]
pub mod claims;
pub mod comment;
pub mod community;
#[cfg(feature = "full")]
pub mod context;
pub mod custom_emoji;
pub mod person;
pub mod post;
pub mod private_message;
#[cfg(feature = "full")]
pub mod request;
#[cfg(feature = "full")]
pub mod send_activity;
pub mod sensitive;
pub mod site;
pub mod tagline;
#[cfg(feature = "full")]
pub mod utils;

pub extern crate lemmy_db_schema;
pub extern crate lemmy_db_views;
pub extern crate lemmy_db_views_actor;
pub extern crate lemmy_db_views_moderator;
pub extern crate lemmy_utils;

pub use lemmy_utils::LemmyErrorType;
use serde::{Deserialize, Serialize};
use std::{cmp::min, time::Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Saves settings for your user.
pub struct SuccessResponse {
  pub success: bool,
}

impl Default for SuccessResponse {
  fn default() -> Self {
    SuccessResponse { success: true }
  }
}

// TODO: use from_days once stabilized
// https://github.com/rust-lang/rust/issues/120301
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// Calculate how long to sleep until next federation send based on how many
/// retries have already happened. Uses exponential backoff with maximum of one day. The first
/// error is ignored.
pub fn federate_retry_sleep_duration(retry_count: i32) -> Duration {
  debug_assert!(retry_count != 0);
  if retry_count == 1 {
    return Duration::from_secs(0);
  }
  let retry_count = retry_count - 1;
  let pow = 1.25_f64.powf(retry_count.into());
  let pow = Duration::try_from_secs_f64(pow).unwrap_or(DAY);
  min(DAY, pow)
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;

  #[test]
  fn test_federate_retry_sleep_duration() {
    assert_eq!(Duration::from_secs(0), federate_retry_sleep_duration(1));
    assert_eq!(
      Duration::new(1, 250000000),
      federate_retry_sleep_duration(2)
    );
    assert_eq!(
      Duration::new(2, 441406250),
      federate_retry_sleep_duration(5)
    );
    assert_eq!(DAY, federate_retry_sleep_duration(100));
  }
}
