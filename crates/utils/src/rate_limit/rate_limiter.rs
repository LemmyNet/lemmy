use crate::{ApiError, IpAddr, LemmyError};
use log::debug;
use std::{collections::HashMap, time::SystemTime};
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
struct RateLimitBucket {
  last_checked: SystemTime,
  allowance: f64,
}

#[derive(Eq, PartialEq, Hash, Debug, EnumIter, Copy, Clone, AsRefStr)]
pub(crate) enum RateLimitType {
  Message,
  Register,
  Post,
  Image,
}

/// Rate limiting based on rate type and IP addr
#[derive(Debug, Clone)]
pub struct RateLimiter {
  buckets: HashMap<RateLimitType, HashMap<IpAddr, RateLimitBucket>>,
}

impl Default for RateLimiter {
  fn default() -> Self {
    Self {
      buckets: HashMap::<RateLimitType, HashMap<IpAddr, RateLimitBucket>>::new(),
    }
  }
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
              last_checked: SystemTime::now(),
              allowance: -2f64,
            },
          );
        }
      }
    }
  }

  #[allow(clippy::float_cmp)]
  pub(super) fn check_rate_limit_full(
    &mut self,
    type_: RateLimitType,
    ip: &IpAddr,
    rate: i32,
    per: i32,
    check_only: bool,
  ) -> Result<(), LemmyError> {
    self.insert_ip(ip);
    if let Some(bucket) = self.buckets.get_mut(&type_) {
      if let Some(rate_limit) = bucket.get_mut(ip) {
        let current = SystemTime::now();
        let time_passed = current.duration_since(rate_limit.last_checked)?.as_secs() as f64;

        // The initial value
        if rate_limit.allowance == -2f64 {
          rate_limit.allowance = rate as f64;
        };

        rate_limit.last_checked = current;
        rate_limit.allowance += time_passed * (rate as f64 / per as f64);
        if !check_only && rate_limit.allowance > rate as f64 {
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
          Err(
            ApiError {
              message: format!(
                "Too many requests. type: {}, IP: {}, {} per {} seconds",
                type_.as_ref(),
                ip,
                rate,
                per
              ),
            }
            .into(),
          )
        } else {
          if !check_only {
            rate_limit.allowance -= 1.0;
          }
          Ok(())
        }
      } else {
        Ok(())
      }
    } else {
      Ok(())
    }
  }
}
