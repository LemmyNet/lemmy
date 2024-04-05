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

// TODO: use from_hours once stabilized
// https://github.com/rust-lang/rust/issues/120301
const HOUR: Duration = Duration::from_secs(3600);

/// Calculate how long to sleep until next federation send based on how many
/// retries have already happened. Uses exponential backoff with maximum of one hour.
pub fn federate_retry_sleep_duration(retry_count: i32) -> Duration {
  let pow = 2.0_f64.powf(retry_count.into());
  let pow = Duration::from_secs_f64(pow);
  min(HOUR, pow)
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;

  #[test]
  fn test_federate_retry_sleep_duration() {
    let s = |secs| Duration::from_secs(secs);
    assert_eq!(s(1), federate_retry_sleep_duration(0));
    assert_eq!(s(2), federate_retry_sleep_duration(1));
    assert_eq!(s(4), federate_retry_sleep_duration(2));
    assert_eq!(s(8), federate_retry_sleep_duration(3));
    assert_eq!(s(16), federate_retry_sleep_duration(4));
    assert_eq!(s(1024), federate_retry_sleep_duration(10));
    assert_eq!(s(3600), federate_retry_sleep_duration(20));
  }
}
