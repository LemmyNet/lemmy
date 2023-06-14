use enum_map::{enum_map, EnumMap};
use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  net::Ipv6Addr,
  time::{Duration, Instant},
};
use tracing::debug;

static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

#[derive(Debug, Clone, Copy)]
struct InstantSecs {
  secs: u32,
}

impl InstantSecs {
  fn now() -> Self {
    InstantSecs {
      secs: u32::try_from(START_TIME.elapsed().as_secs())
        .expect("server has been running for over 136 years"),
    }
  }

  fn secs_since(self, earlier: Self) -> u32 {
    self.secs.saturating_sub(earlier.secs)
  }

  fn to_instant(self) -> Instant {
    *START_TIME + Duration::from_secs(self.secs.into())
  }
}

#[derive(Debug, Clone)]
struct RateLimitBucket {
  last_checked: InstantSecs,
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
  buckets: HashMap<Ipv6Addr, EnumMap<RateLimitType, RateLimitBucket>>,
}

impl RateLimitStorage {
  /// Rate limiting Algorithm described here: https://stackoverflow.com/a/668327/1655478
  ///
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub(super) fn check_rate_limit_full(
    &mut self,
    type_: RateLimitType,
    ip: &Ipv6Addr,
    rate: i32,
    per: i32,
  ) -> bool {
    let current = InstantSecs::now();
    let ip_buckets = self.buckets.entry(*ip).or_insert(enum_map! {
      _ => RateLimitBucket {
        last_checked: current,
        allowance: rate as f32,
      },
    });
    #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
    let rate_limit = &mut ip_buckets[type_];
    let time_passed = current.secs_since(rate_limit.last_checked) as f32;

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

    self.buckets.retain(|_ip_addr, buckets| {
      buckets
        .values()
        .all(|bucket| bucket.last_checked.to_instant() > instant)
    });
  }
}
