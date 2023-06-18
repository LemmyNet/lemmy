use enum_map::{enum_map, EnumMap};
use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
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

type Map<K, C> = HashMap<K, RateLimitedGroup<C>>;

#[derive(Debug, Clone)]
struct RateLimitedGroup<C> {
  total: EnumMap<RateLimitType, RateLimitBucket>,
  children: C,
}

impl<C: Default> RateLimitedGroup<C> {
  fn new(now: InstantSecs) -> Self {
    RateLimitedGroup {
      total: enum_map! {
        _ => RateLimitBucket {
          last_checked: now,
          tokens: -2.0,
        },
      },
      children: Default::default(),
    }
  }

  fn check_total(
    &mut self,
    type_: RateLimitType,
    now: InstantSecs,
    capacity: i32,
    secs_to_refill: i32,
  ) -> bool {
    let capacity = capacity as f32;
    let secs_to_refill = secs_to_refill as f32;

    #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
    let bucket = &mut self.total[type_];

    if bucket.tokens == -2.0 {
      bucket.tokens = capacity;
    }

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
        "Rate limited type: {}, time_passed: {}, allowance: {}",
        type_.as_ref(),
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
}

/// Rate limiting based on rate type and IP addr
#[derive(Debug, Clone, Default)]
pub struct RateLimitStorage {
  /// One bucket per individual IPv4 address
  ipv4_buckets: Map<Ipv4Addr, ()>,
  /// Seperate buckets for 48, 56, and 64 bit prefixes of IPv6 addresses
  ipv6_buckets: Map<[u8; 6], Map<u8, Map<u8, ()>>>,
}

impl RateLimitStorage {
  /// Rate limiting Algorithm described here: https://stackoverflow.com/a/668327/1655478
  ///
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub(super) fn check_rate_limit_full(
    &mut self,
    type_: RateLimitType,
    ip: IpAddr,
    capacity: i32,
    secs_to_refill: i32,
  ) -> bool {
    let now = InstantSecs::now();

    let mut result = true;

    match ip {
      IpAddr::V4(ipv4) => {
        let group = self
          .ipv4_buckets
          .entry(ipv4)
          .or_insert(RateLimitedGroup::new(now));

        result &= group.check_total(type_, now, capacity, secs_to_refill);
      }

      IpAddr::V6(ipv6) => {
        let (key_48, key_56, key_64) = split_ipv6(ipv6);

        let group_48 = self
          .ipv6_buckets
          .entry(key_48)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_48.check_total(type_, now, capacity.saturating_mul(16), secs_to_refill);

        let group_56 = group_48
          .children
          .entry(key_56)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_56.check_total(type_, now, capacity.saturating_mul(4), secs_to_refill);

        let group_64 = group_56
          .children
          .entry(key_64)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_64.check_total(type_, now, capacity, secs_to_refill);
      }
    }

    if !result {
      debug!("Rate limited IP: {ip}");
    }

    result
  }

  /// Remove buckets older than the given duration
  pub(super) fn remove_older_than(&mut self, duration: Duration) {
    // Only retain buckets that were last used after `instant`
    let Some(instant) = Instant::now().checked_sub(duration) else { return };

    let is_recently_used = |group: &RateLimitedGroup<_>| {
      group
        .total
        .values()
        .all(|bucket| bucket.last_checked.to_instant() > instant)
    };

    self.ipv4_buckets.retain(|_, group| is_recently_used(group));

    self.ipv6_buckets.retain(|_, group_48| {
      group_48.children.retain(|_, group_56| {
        group_56
          .children
          .retain(|_, group_64| is_recently_used(group_64));
        !group_56.children.is_empty()
      });
      !group_48.children.is_empty()
    })
  }
}

fn split_ipv6(ip: Ipv6Addr) -> ([u8; 6], u8, u8) {
  let [a0, a1, a2, a3, a4, a5, b, c, ..] = ip.octets();
  ([a0, a1, a2, a3, a4, a5], b, c)
}
