use enum_map::{enum_map, EnumMap};
use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  hash::Hash,
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
  time::{Duration, Instant},
};
use tracing::debug;

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

/// Represents a bucket that holds an amount of tokens. `BucketConfig` determines how
/// the amount of tokens grows. The bucket starts with `capacity` tokens, and performing
/// the rate-limited action consumes 1 token. The amount of tokens gradually returns to
/// `capacity` at the rate of `capacity` tokens every `secs_to_refill` seconds. So when
/// there's 0 tokens, it will take `secs_to_refill` seconds for `capacity` tokens to
/// reappear.
#[derive(PartialEq, Debug, Clone)]
struct RateLimitBucket {
  /// This is the time at which the amount of tokens becomes `capacity`. This can be used
  /// to calculate the amount of tokens at any given time after the previous token
  /// consumption if the `BucketConfig` is known.
  refill_time: InstantSecs,
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

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct BucketConfig {
  pub capacity: i32,
  pub secs_to_refill: i32,
}

impl BucketConfig {
  fn multiply_capacity(self, rhs: i32) -> Self {
    BucketConfig {
      capacity: self.capacity.saturating_mul(rhs),
      ..self
    }
  }
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
          refill_time: now
        },
      },
      children: Default::default(),
    }
  }

  fn check_total(&mut self, type_: RateLimitType, now: InstantSecs, config: BucketConfig) -> bool {
    let capacity = config.capacity as f32;
    let secs_to_refill = config.secs_to_refill as f32;

    #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
    let bucket = &mut self.total[type_];

    // 0 seconds if bucket is already full
    let remaining_secs_until_refill = bucket.refill_time.secs_since(now) as f32;

    let tokens = capacity * (1.0 - (remaining_secs_until_refill / secs_to_refill));

    if tokens < 1.0 {
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
      let secs_to_add_1_token = (secs_to_refill / capacity).ceil() as u32;
      bucket.refill_time.secs = bucket.refill_time.secs.saturating_add(secs_to_add_1_token);
      true
    }
  }
}

/// Rate limiting based on rate type and IP addr
#[derive(PartialEq, Debug, Clone)]
pub struct RateLimitStorage {
  /// One bucket per individual IPv4 address
  ipv4_buckets: Map<Ipv4Addr, ()>,
  /// Seperate buckets for 48, 56, and 64 bit prefixes of IPv6 addresses
  ipv6_buckets: Map<[u8; 6], Map<u8, Map<u8, ()>>>,
  bucket_configs: EnumMap<RateLimitType, BucketConfig>,
}

impl RateLimitStorage {
  pub fn new(bucket_configs: EnumMap<RateLimitType, BucketConfig>) -> Self {
    RateLimitStorage {
      ipv4_buckets: Default::default(),
      ipv6_buckets: Default::default(),
      bucket_configs,
    }
  }

  /// Rate limiting Algorithm described here: https://stackoverflow.com/a/668327/1655478
  ///
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub(super) fn check_rate_limit_full(
    &mut self,
    type_: RateLimitType,
    ip: IpAddr,
    now: InstantSecs,
  ) -> bool {
    #[allow(clippy::indexing_slicing)]
    let config = self.bucket_configs[type_];
    let mut result = true;

    match ip {
      IpAddr::V4(ipv4) => {
        // Only used by one address.
        let group = self
          .ipv4_buckets
          .entry(ipv4)
          .or_insert(RateLimitedGroup::new(now));

        result &= group.check_total(type_, now, config);
      }

      IpAddr::V6(ipv6) => {
        let (key_48, key_56, key_64) = split_ipv6(ipv6);

        // Contains all addresses with the same first 48 bits. These addresses might be part of the same network.
        let group_48 = self
          .ipv6_buckets
          .entry(key_48)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_48.check_total(type_, now, config.multiply_capacity(16));

        // Contains all addresses with the same first 56 bits. These addresses might be part of the same network.
        let group_56 = group_48
          .children
          .entry(key_56)
          .or_insert(RateLimitedGroup::new(now));
        result &= group_56.check_total(type_, now, config.multiply_capacity(4));

        // A group with no children. It is shared by all addresses with the same first 64 bits. These addresses are always part of the same network.
        let group_64 = group_56
          .children
          .entry(key_64)
          .or_insert(RateLimitedGroup::new(now));

        result &= group_64.check_total(type_, now, config);
      }
    };

    if !result {
      debug!("Rate limited IP: {ip}");
    }

    result
  }

  /// Remove buckets that are now full
  pub(super) fn remove_full_buckets(&mut self, now: InstantSecs) {
    // Only retain buckets that were last used after `instant`
    let Some(instant) = now.to_instant().checked_sub(duration) else {
      return;
    };

    let is_recently_used = |group: &RateLimitedGroup<_>| {
      group
        .total
        .iter()
        .all(|(type_, bucket)| {
          // Refill is in the future
          bucket.refill_time.to_instant() > now.to_instant()
        })
    };

    retain_and_shrink(&mut self.ipv4_buckets, |_, group| is_recently_used(group));

    retain_and_shrink(&mut self.ipv6_buckets, |_, group_48| {
      retain_and_shrink(&mut group_48.children, |_, group_56| {
        retain_and_shrink(&mut group_56.children, |_, group_64| {
          is_recently_used(group_64)
        });
        !group_56.children.is_empty() || is_recently_used(group_56.total)
      });
      !group_48.children.is_empty() || is_recently_used(group_48.total)
    })
  }
}

fn retain_and_shrink<K, V, F>(map: &mut HashMap<K, V>, f: F)
where
  K: Eq + Hash,
  F: FnMut(&K, &mut V) -> bool,
{
  map.retain(f);
  map.shrink_to_fit();
}

fn split_ipv6(ip: Ipv6Addr) -> ([u8; 6], u8, u8) {
  let [a0, a1, a2, a3, a4, a5, b, c, ..] = ip.octets();
  ([a0, a1, a2, a3, a4, a5], b, c)
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

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
    let mut rate_limiter = super::RateLimitStorage::new(enum_map! {
      super::RateLimitType::Message => super::BucketConfig {
        capacity: 2,
        secs_to_refill: 1,
      },
      _ => super::BucketConfig {
        capacity: 2,
        secs_to_refill: 1,
      },
    });
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
        rate_limiter.check_rate_limit_full(super::RateLimitType::Message, ip, now);
      let post_passed =
        rate_limiter.check_rate_limit_full(super::RateLimitType::Post, ip, now);
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
