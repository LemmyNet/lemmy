use enum_map::{enum_map, EnumMap};
use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
  time::{Duration, Instant},
};
use tracing::debug;

const UNINITIALIZED_TOKEN_AMOUNT: f32 = -2.0;

static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

/// Smaller than `std::time::Instant` because it uses a smaller integer for seconds and doesn't
/// store nanoseconds
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct InstantSecs {
  secs: u32,
}

impl InstantSecs {
  pub fn now() -> Self {
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

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
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
          tokens: UNINITIALIZED_TOKEN_AMOUNT,
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

    if bucket.tokens == UNINITIALIZED_TOKEN_AMOUNT {
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
#[derive(PartialEq, Debug, Clone, Default)]
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
    now: InstantSecs,
  ) -> bool {
    let mut result = true;

    match ip {
      IpAddr::V4(ipv4) => {
        // Only used by one address.
        let group = self
          .ipv4_buckets
          .entry(ipv4)
          .or_insert(RateLimitedGroup::new(now));

        result &= group.check_total(type_, now, capacity, secs_to_refill);
      }

      IpAddr::V6(ipv6) => {
        let (key_48, key_56, key_64) = split_ipv6(ipv6);

        // Contains all addresses with the same first 48 bits. These addresses might be part of the same network.
        let group_48 = self
          .ipv6_buckets
          .entry(key_48)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_48.check_total(type_, now, capacity.saturating_mul(16), secs_to_refill);

        // Contains all addresses with the same first 56 bits. These addresses might be part of the same network.
        let group_56 = group_48
          .children
          .entry(key_56)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_56.check_total(type_, now, capacity.saturating_mul(4), secs_to_refill);

        // A group with no children. It is shared by all addresses with the same first 64 bits. These addresses are always part of the same network.
        let group_64 = group_56
          .children
          .entry(key_64)
          .or_insert(RateLimitedGroup::new(now));

        result &= group_64.check_total(type_, now, capacity, secs_to_refill);
      }
    };

    if !result {
      debug!("Rate limited IP: {ip}");
    }

    result
  }

  /// Remove buckets older than the given duration
  pub(super) fn remove_older_than(&mut self, duration: Duration, now: InstantSecs) {
    // Only retain buckets that were last used after `instant`
    let Some(instant) = now.to_instant().checked_sub(duration) else { return };

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

#[cfg(test)]
mod tests {
  #[test]
  fn test_split_ipv6() {
    let ip = std::net::Ipv6Addr::new(
      0x0011, 0x2233, 0x4455, 0x6677, 0x8899, 0xAABB, 0xCCDD, 0xEEFF,
    );
    assert_eq!(
      super::split_ipv6(ip),
      ([0x00, 0x11, 0x22, 0x33, 0x44, 0x55], 0x66, 0x77)
    );
  }

  #[test]
  fn test_rate_limiter() {
    let mut rate_limiter = super::RateLimitStorage::default();
    let mut now = super::InstantSecs::now();

    let ips = [
      "123.123.123.123",
      "1:2:3::",
      "1:2:3:0400::",
      "1:2:3:0405::",
      "1:2:3:0405:6::",
    ];
    for ip in ips {
      let ip = ip.parse().unwrap();
      let message_passed =
        rate_limiter.check_rate_limit_full(super::RateLimitType::Message, ip, 2, 1, now);
      let post_passed =
        rate_limiter.check_rate_limit_full(super::RateLimitType::Post, ip, 3, 1, now);
      assert!(message_passed);
      assert!(post_passed);
    }

    #[allow(clippy::indexing_slicing)]
    let expected_buckets = |factor: f32, tokens_consumed: f32| {
      let mut buckets = super::RateLimitedGroup::<()>::new(now).total;
      buckets[super::RateLimitType::Message] = super::RateLimitBucket {
        last_checked: now,
        tokens: (2.0 * factor) - tokens_consumed,
      };
      buckets[super::RateLimitType::Post] = super::RateLimitBucket {
        last_checked: now,
        tokens: (3.0 * factor) - tokens_consumed,
      };
      buckets
    };

    let bottom_group = |tokens_consumed| super::RateLimitedGroup {
      total: expected_buckets(1.0, tokens_consumed),
      children: (),
    };

    assert_eq!(
      rate_limiter,
      super::RateLimitStorage {
        ipv4_buckets: [([123, 123, 123, 123].into(), bottom_group(1.0)),].into(),
        ipv6_buckets: [(
          [0, 1, 0, 2, 0, 3],
          super::RateLimitedGroup {
            total: expected_buckets(16.0, 4.0),
            children: [
              (
                0,
                super::RateLimitedGroup {
                  total: expected_buckets(4.0, 1.0),
                  children: [(0, bottom_group(1.0)),].into(),
                }
              ),
              (
                4,
                super::RateLimitedGroup {
                  total: expected_buckets(4.0, 3.0),
                  children: [(0, bottom_group(1.0)), (5, bottom_group(2.0)),].into(),
                }
              ),
            ]
            .into(),
          }
        ),]
        .into(),
      }
    );

    now.secs += 2;
    rate_limiter.remove_older_than(std::time::Duration::from_secs(1), now);
    assert!(rate_limiter.ipv4_buckets.is_empty());
    assert!(rate_limiter.ipv6_buckets.is_empty());
  }
}
