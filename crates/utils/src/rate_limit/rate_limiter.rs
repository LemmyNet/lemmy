use enum_map::{enum_map, EnumMap};
use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  net::Ipv6Addr,
  time::{Duration, Instant},
};
use tracing::debug;

static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

/// Smaller than `std::time::Instant` because it uses a smaller integer for seconds and doesn't
/// store nanoseconds
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
  /// This field stores the amount of tokens that were present at `last_checked`.
  /// The amount of tokens steadily increases until it reaches the bucket's capacity.
  /// Performing the rate-limited action consumes 1 token.
  tokens: f32,
}

#[derive(Debug, enum_map::Enum, Copy, Clone, AsRefStr)]
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
    capacity: i32,
    secs_to_refill: i32,
  ) -> bool {
    let capacity = capacity as f32;
    let secs_to_refill = secs_to_refill as f32;

    let now = InstantSecs::now();
    let bucket = {
      let default = enum_map! {
        _ => RateLimitBucket {
          last_checked: now,
          tokens: capacity,
        },
      };
      let ip_buckets = self.buckets.entry(*ip).or_insert(default);
      #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
      &mut ip_buckets[type_]
    };

    let secs_since_last_checked = now.secs_since(bucket.last_checked) as f32;
    bucket.last_checked = now;

    // For `secs_since_last_checked` seconds, increase `bucket.tokens`
    // by `capacity` every `secs_to_refill` seconds
    bucket.tokens += {
      let tokens_per_sec = capacity / secs_to_refill;
      secs_since_last_checked * tokens_per_sec
    };

    // Prevent `bucket.tokens` from exceeding `capacity`
    if bucket.tokens > capacity {
      bucket.tokens = capacity;
    }

    if bucket.tokens < 1.0 {
      // Not enough tokens yet
      debug!(
        "Rate limited type: {}, IP: {}, time_passed: {}, allowance: {}",
        type_.as_ref(),
        ip,
        secs_since_last_checked,
        bucket.tokens
      );
      false
    } else {
      // Consume 1 token
      bucket.tokens -= 1.0;
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
