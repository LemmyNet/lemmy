use enum_map::EnumMap;
use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  hash::Hash,
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
  time::Instant,
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
}

#[derive(PartialEq, Debug, Clone, Copy)]
struct Bucket {
  last_checked: InstantSecs,
  /// This field stores the amount of tokens that were present at `last_checked`.
  /// The amount of tokens steadily increases until it reaches the bucket's capacity.
  /// Performing the rate-limited action consumes 1 token.
  tokens: u32,
}

impl Bucket {
  fn update(self, now: InstantSecs, config: BucketConfig) -> Self {
    let secs_since_last_checked = now.secs.saturating_sub(self.last_checked.secs);

    // For `secs_since_last_checked` seconds, the amount of tokens increases by `capacity` every `secs_to_refill` seconds.
    // The amount of tokens added per second is `capacity / secs_to_refill`.
    // The expression below is like `secs_since_last_checked * (capacity / secs_to_refill)` but with precision and non-overflowing multiplication.
    let added_tokens = u64::from(secs_since_last_checked) * u64::from(config.capacity)
      / u64::from(config.secs_to_refill);

    // The amount of tokens there would be if the bucket had infinite capacity
    let unbounded_tokens = self.tokens + (added_tokens as u32);

    // Bucket stops filling when capacity is reached
    let tokens = std::cmp::min(unbounded_tokens, config.capacity);

    Bucket {
      last_checked: now,
      tokens,
    }
  }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct BucketConfig {
  pub capacity: u32,
  pub secs_to_refill: u32,
}

#[derive(Debug, enum_map::Enum, Copy, Clone, AsRefStr)]
pub enum ActionType {
  Message,
  Register,
  Post,
  Image,
  Comment,
  Search,
  ImportUserSettings,
}

#[derive(PartialEq, Debug, Clone)]
struct RateLimitedGroup<C> {
  total: EnumMap<ActionType, Bucket>,
  children: C,
}

impl<C: Default> RateLimitedGroup<C> {
  fn new(now: InstantSecs, configs: EnumMap<ActionType, BucketConfig>) -> Self {
    RateLimitedGroup {
      total: configs.map(|_, config| Bucket {
        last_checked: now,
        tokens: config.capacity,
      }),
      // `HashMap::new()` or `()`
      children: Default::default(),
    }
  }

  fn check_total(
    &mut self,
    action_type: ActionType,
    now: InstantSecs,
    config: BucketConfig,
  ) -> bool {
    #[allow(clippy::indexing_slicing)] // `EnumMap` has no `get` funciton
    let bucket = &mut self.total[action_type];

    let new_bucket = bucket.update(now, config);

    if new_bucket.tokens == 0 {
      // Not enough tokens yet
      // Setting `bucket` to `new_bucket` here is useless and would cause the bucket to start over at 0 tokens because of rounding
      false
    } else {
      // Consume 1 token
      *bucket = new_bucket;
      bucket.tokens -= 1;
      true
    }
  }
}

type Map<K, C> = HashMap<K, RateLimitedGroup<C>>;

/// Implemented for `()`, `Map<T, ()>`, `Map<T, Map<U, ()>>`, etc.
trait MapLevel: Default {
  type CapacityFactors;
  type AddrParts;

  fn check(
    &mut self,
    action_type: ActionType,
    now: InstantSecs,
    configs: EnumMap<ActionType, BucketConfig>,
    capacity_factors: Self::CapacityFactors,
    addr_parts: Self::AddrParts,
  ) -> bool;

  /// Remove full buckets and return `true` if there's any buckets remaining
  fn remove_full_buckets(
    &mut self,
    now: InstantSecs,
    configs: EnumMap<ActionType, BucketConfig>,
  ) -> bool;
}

impl<K: Eq + Hash, C: MapLevel> MapLevel for Map<K, C> {
  type CapacityFactors = (u32, C::CapacityFactors);
  type AddrParts = (K, C::AddrParts);

  fn check(
    &mut self,
    action_type: ActionType,
    now: InstantSecs,
    configs: EnumMap<ActionType, BucketConfig>,
    (capacity_factor, child_capacity_factors): Self::CapacityFactors,
    (addr_part, child_addr_parts): Self::AddrParts,
  ) -> bool {
    // Multiplies capacities by `capacity_factor` for groups in `self`
    let adjusted_configs = configs.map(|_, config| BucketConfig {
      capacity: config.capacity.saturating_mul(capacity_factor),
      ..config
    });

    let group = self
      .entry(addr_part)
      .or_insert(RateLimitedGroup::new(now, adjusted_configs));

    #[allow(clippy::indexing_slicing)]
    let total_passes = group.check_total(action_type, now, adjusted_configs[action_type]);

    let children_pass = group.children.check(
      action_type,
      now,
      configs,
      child_capacity_factors,
      child_addr_parts,
    );

    total_passes && children_pass
  }

  fn remove_full_buckets(
    &mut self,
    now: InstantSecs,
    configs: EnumMap<ActionType, BucketConfig>,
  ) -> bool {
    self.retain(|_key, group| {
      let some_children_remaining = group.children.remove_full_buckets(now, configs);

      // Evaluated if `some_children_remaining` is false
      let total_has_refill_in_future = || {
        group.total.into_iter().all(|(action_type, bucket)| {
          #[allow(clippy::indexing_slicing)]
          let config = configs[action_type];
          bucket.update(now, config).tokens != config.capacity
        })
      };

      some_children_remaining || total_has_refill_in_future()
    });

    self.shrink_to_fit();

    !self.is_empty()
  }
}

impl MapLevel for () {
  type CapacityFactors = ();
  type AddrParts = ();

  fn check(
    &mut self,
    _action_type: ActionType,
    _now: InstantSecs,
    _configs: EnumMap<ActionType, BucketConfig>,
    _capacity_factors: Self::CapacityFactors,
    _addr_parts: Self::AddrParts,
  ) -> bool {
    true
  }

  fn remove_full_buckets(
    &mut self,
    _now: InstantSecs,
    _configs: EnumMap<ActionType, BucketConfig>,
  ) -> bool {
    false
  }
}

/// Rate limiting based on rate type and IP addr
#[derive(PartialEq, Debug, Clone)]
pub struct RateLimitStorage {
  /// Each individual IPv4 address gets a bucket.
  ipv4_buckets: Map<Ipv4Addr, ()>,
  /// All IPv6 addresses that share the same first 64 bits share the same bucket.
  ///
  /// The same thing happens for the first 48 and 56 bits, but with increased capacity.
  ///
  /// This is done because all users can easily switch to any other IPv6 address that has the same 64 bits.
  /// Sometimes they can do the same thing but with even less bits staying the same, which is the reason for
  /// 48 and 56 bit address groups.
  ipv6_buckets: Map<[u8; 6], Map<u8, Map<u8, ()>>>,
  bucket_configs: EnumMap<ActionType, BucketConfig>,
}

impl RateLimitStorage {
  pub fn new(bucket_configs: EnumMap<ActionType, BucketConfig>) -> Self {
    RateLimitStorage {
      ipv4_buckets: HashMap::new(),
      ipv6_buckets: HashMap::new(),
      bucket_configs,
    }
  }

  /// Rate limiting Algorithm described here: https://stackoverflow.com/a/668327/1655478
  ///
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub fn check(&mut self, action_type: ActionType, ip: IpAddr, now: InstantSecs) -> bool {
    let result = match ip {
      IpAddr::V4(ipv4) => {
        self
          .ipv4_buckets
          .check(action_type, now, self.bucket_configs, (1, ()), (ipv4, ()))
      }

      IpAddr::V6(ipv6) => {
        let (key_48, key_56, key_64) = split_ipv6(ipv6);
        self.ipv6_buckets.check(
          action_type,
          now,
          self.bucket_configs,
          (16, (4, (1, ()))),
          (key_48, (key_56, (key_64, ()))),
        )
      }
    };

    if !result {
      debug!("Rate limited IP: {ip}, type: {action_type:?}");
    }

    result
  }

  /// Remove buckets that are now full
  pub fn remove_full_buckets(&mut self, now: InstantSecs) {
    self
      .ipv4_buckets
      .remove_full_buckets(now, self.bucket_configs);
    self
      .ipv6_buckets
      .remove_full_buckets(now, self.bucket_configs);
  }

  pub fn set_config(&mut self, new_configs: EnumMap<ActionType, BucketConfig>) {
    self.bucket_configs = new_configs;
  }
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
    let bucket_configs = enum_map::enum_map! {
      super::ActionType::Message => super::BucketConfig {
        capacity: 2,
        secs_to_refill: 1,
      },
      _ => super::BucketConfig {
        capacity: 2,
        secs_to_refill: 1,
      },
    };
    let mut rate_limiter = super::RateLimitStorage::new(bucket_configs);
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
      let message_passed = rate_limiter.check(super::ActionType::Message, ip, now);
      let post_passed = rate_limiter.check(super::ActionType::Post, ip, now);
      assert!(message_passed);
      assert!(post_passed);
    }

    #[allow(clippy::indexing_slicing)]
    let expected_buckets = |factor: u32, tokens_consumed: u32| {
      let mut buckets = super::RateLimitedGroup::<()>::new(now, bucket_configs).total;
      buckets[super::ActionType::Message] = super::Bucket {
        last_checked: now,
        tokens: (2 * factor) - tokens_consumed,
      };
      buckets[super::ActionType::Post] = super::Bucket {
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
        bucket_configs,
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
    rate_limiter.remove_full_buckets(now);
    assert!(rate_limiter.ipv4_buckets.is_empty());
    assert!(rate_limiter.ipv6_buckets.is_empty());
  }
}
