use crate::IpAddr;
use std::{collections::HashMap, time::Instant};
use strum::IntoEnumIterator;
use tracing::debug;

#[derive(Debug, Clone)]
struct RateLimitBucket {
  last_checked: Instant,
  allowance: f64,
}

#[derive(Eq, PartialEq, Hash, Debug, EnumIter, Copy, Clone, AsRefStr)]
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
pub struct RateLimiter {
  buckets: HashMap<RateLimitType, HashMap<IpAddr, RateLimitBucket>>,
}

impl RateLimiter {
  fn insert_ip(&mut self, ip: &IpAddr) {
    for rate_limit_type in RateLimitType::iter() {
      if self.buckets.get(&rate_limit_type).is_none() {
        self.buckets.insert(rate_limit_type, HashMap::new());
      }

      if let Some(bucket) = self.buckets.get_mut(&rate_limit_type) {
        if bucket.get(ip).is_none() {
          bucket.insert(
            ip.clone(),
            RateLimitBucket {
              last_checked: Instant::now(),
              allowance: -2f64,
            },
          );
        }
      }
    }
  }

  /// Rate limiting Algorithm described here: https://stackoverflow.com/a/668327/1655478
  ///
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  #[allow(clippy::float_cmp)]
  pub(super) fn check_rate_limit_full(
    &mut self,
    type_: RateLimitType,
    ip: &IpAddr,
    rate: i32,
    per: i32,
  ) -> bool {
    self.insert_ip(ip);
    if let Some(bucket) = self.buckets.get_mut(&type_) {
      if let Some(rate_limit) = bucket.get_mut(ip) {
        let current = Instant::now();
        let time_passed = current.duration_since(rate_limit.last_checked).as_secs() as f64;

        // The initial value
        if rate_limit.allowance == -2f64 {
          rate_limit.allowance = rate as f64;
        };

        rate_limit.last_checked = current;
        rate_limit.allowance += time_passed * (rate as f64 / per as f64);
        if rate_limit.allowance > rate as f64 {
          rate_limit.allowance = rate as f64;
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
      } else {
        true
      }
    } else {
      true
    }
  }
}
