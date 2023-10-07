use enum_map::EnumMap;
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
}

#[derive(PartialEq, Debug, Clone, Copy)]
struct RateLimitBucket {
  last_checked: InstantSecs,
  /// This field stores the amount of tokens that were present at `last_checked`.
  /// The amount of tokens steadily increases until it reaches the bucket's capacity.
  /// Performing the rate-limited action consumes 1 token.
  tokens: i32,
}

impl RateLimitBucket {
  fn update(mut self, now: InstantSecs, config: BucketConfig) -> Self {
    let secs_since_last_checked = now.secs_since(self.last_checked);
    self.last_checked = now;

    // For `secs_since_last_checked` seconds, increase `self.tokens` by `capacity` every `secs_to_refill` seconds.
    // Amount of tokens added per second is `capacity / secs_to_refill`.
    // The added expression below is like `secs_since_last_checked * (capacity / secs_to_refill)` but with precision and less chance of integer overflow.
    self.tokens += (i64::from(secs_since_last_checked) * i64::from(config.capacity)
      / i64::from(config.secs_to_refill)) as i32;

    // Prevent `self.tokens` from exceeding `capacity`
    if self.tokens > config.capacity {
      self.tokens = config.capacity;
    }

    self
  }
}

#[derive(Debug, enum_map::Enum, Copy, Clone, AsRefStr)]
pub enum RateLimitType {
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
  fn new(now: InstantSecs, configs: EnumMap<RateLimitType, BucketConfig>) -> Self {
    RateLimitedGroup {
      total: configs.map(|_, config| RateLimitBucket {
        last_checked: now,
        tokens: config.capacity,
      }),
      children: Default::default(),
    }
  }

  fn check_total(&mut self, type_: RateLimitType, now: InstantSecs, config: BucketConfig) -> bool {
    #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
    let bucket = &mut self.total[type_];

    let updated_bucket = bucket.update(now, config);

    if updated_bucket.tokens < 1 {
      // Not enough tokens yet
      debug!(
        "Rate limited type: {}, allowance: {}",
        type_.as_ref(),
        updated_bucket.tokens
      );
      false
    } else {
      // Consume 1 token
      *bucket = updated_bucket;
      bucket.tokens -= 1;
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
  pub fn check_rate_limit_full(
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
          .or_insert(RateLimitedGroup::new(now, self.bucket_configs));

        result &= group.check_total(type_, now, config);
      }

      IpAddr::V6(ipv6) => {
        let (key_48, key_56, key_64) = split_ipv6(ipv6);

        // Contains all addresses with the same first 48 bits. These addresses might be part of the same network.
        let group_48 = self
          .ipv6_buckets
          .entry(key_48)
          .or_insert(RateLimitedGroup::new(now, self.bucket_configs));
        result &= group_48.check_total(type_, now, config.multiply_capacity(16));

        // Contains all addresses with the same first 56 bits. These addresses might be part of the same network.
        let group_56 = group_48
          .children
          .entry(key_56)
          .or_insert(RateLimitedGroup::new(now, self.bucket_configs));
        result &= group_56.check_total(type_, now, config.multiply_capacity(4));

        // A group with no children. It is shared by all addresses with the same first 64 bits. These addresses are always part of the same network.
        let group_64 = group_56
          .children
          .entry(key_64)
          .or_insert(RateLimitedGroup::new(now, self.bucket_configs));

        result &= group_64.check_total(type_, now, config);
      }
    };

    if !result {
      debug!("Rate limited IP: {ip}");
    }

    result
  }

  /// Remove buckets that are now full
  pub fn remove_full_buckets(&mut self, now: InstantSecs) {
    let has_refill_in_future = |buckets: EnumMap<RateLimitType, RateLimitBucket>| {
      buckets.iter().all(|(type_, bucket)| {
        #[allow(clippy::indexing_slicing)]
        let config = self.bucket_configs[type_];
        bucket.update(now, config).tokens != config.capacity
      })
    };

    retain_and_shrink(&mut self.ipv4_buckets, |_, group| {
      has_refill_in_future(group.total)
    });

    retain_and_shrink(&mut self.ipv6_buckets, |_, group_48| {
      retain_and_shrink(&mut group_48.children, |_, group_56| {
        retain_and_shrink(&mut group_56.children, |_, group_64| {
          has_refill_in_future(group_64.total)
        });
        !group_56.children.is_empty() || has_refill_in_future(group_56.total)
      });
      !group_48.children.is_empty() || has_refill_in_future(group_48.total)
    })
  }

  pub fn set_config(&mut self, new_configs: EnumMap<RateLimitType, BucketConfig>) {
    self.bucket_configs = new_configs;
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
      let post_passed = rate_limiter.check_rate_limit_full(super::RateLimitType::Post, ip, now);
      assert!(message_passed);
      assert!(post_passed);
    }

    #[allow(clippy::indexing_slicing)]
    let expected_buckets = |factor: i32, tokens_consumed: i32| {
      let mut buckets = super::RateLimitedGroup::<()>::new(now).total;
      buckets[super::RateLimitType::Message] = super::RateLimitBucket {
        last_checked: now,
        tokens: (2 * factor) - tokens_consumed,
      };
      buckets[super::RateLimitType::Post] = super::RateLimitBucket {
        last_checked: now,
        tokens: (3 * factor) - tokens_consumed,
      };
      buckets
    };

    let bottom_group = |tokens_consumed| super::RateLimitedGroup {
      total: expected_buckets(1, tokens_consumed),
      children: (),
    };

    assert_eq!(
      rate_limiter,
      super::RateLimitStorage {
        ipv4_buckets: [([123, 123, 123, 123].into(), bottom_group(1)),].into(),
        ipv6_buckets: [(
          [0, 1, 0, 2, 0, 3],
          super::RateLimitedGroup {
            total: expected_buckets(16, 4),
            children: [
              (
                0,
                super::RateLimitedGroup {
                  total: expected_buckets(4, 1),
                  children: [(0, bottom_group(1)),].into(),
                }
              ),
              (
                4,
                super::RateLimitedGroup {
                  total: expected_buckets(4, 3),
                  children: [(0, bottom_group(1)), (5, bottom_group(2)),].into(),
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
