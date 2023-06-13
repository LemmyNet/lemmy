use crate::IpAddr;
use enum_map::{enum_map, EnumMap};
use std::{
  collections::HashMap,
  time::{Duration, Instant},
};
use tracing::debug;

#[derive(Debug, Clone)]
struct RateLimitBucket {
  last_checked: Instant,
  allowance: f32,
}

#[derive(Eq, PartialEq, Hash, Debug, enum_map::Enum, Copy, Clone, AsRefStr)]
pub(crate) enum RateLimitType {
  Message,
  Register,
  Post,
  Image,
  Comment,
  Search,
}

/// Rate limiting based on rate type and IP addr
#[derive(Debug, Clone, Default)]
pub struct RateLimitStorage {
  buckets: HashMap<IpAddr, EnumMap<RateLimitType, RateLimitBucket>>,
}

impl RateLimitStorage {
  /// Rate limiting Algorithm described here: https://stackoverflow.com/a/668327/1655478
  ///
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub(super) fn check_rate_limit_full(
    &mut self,
    type_: RateLimitType,
    ip: &IpAddr,
    rate: i32,
    per: i32,
  ) -> bool {
    let current = Instant::now();
    let ip_buckets = self.buckets.entry(ip.clone()).or_insert(enum_map! {
      _ => RateLimitBucket {
        last_checked: current,
        allowance: rate as f32,
      },
    });
    #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
    let rate_limit = &mut ip_buckets[type_];
    let time_passed = current.duration_since(rate_limit.last_checked).as_secs() as f32;

    rate_limit.last_checked = current;
    rate_limit.allowance += time_passed * (rate as f32 / per as f32);
    if rate_limit.allowance > rate as f32 {
      rate_limit.allowance = rate as f32;
    }

    if rate_limit.allowance < 1.0 {
      debug!(
        "Rate limited type: {}, IP: {}, time_passed: {}, allowance: {}",
        type_.as_ref(),
        ip,
        time_passed,
        rate_limit.allowance
      );
      false
    } else {
      rate_limit.allowance -= 1.0;
      true
    }
  }

  /// Remove buckets older than the given duration
  pub(super) fn remove_older_than(&mut self, duration: Duration) {
    // Only retain buckets that were last used after `instant`
    let Some(instant) = Instant::now().checked_sub(duration) else { return };

    self
      .buckets
      .retain(|_ip_addr, buckets| buckets.values().all(|bucket| bucket.last_checked > instant));
  }
}
