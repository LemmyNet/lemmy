use super::*;

#[derive(Debug, Clone)]
pub struct RateLimitBucket {
  last_checked: SystemTime,
  allowance: f64,
}

#[derive(Eq, PartialEq, Hash, Debug, EnumIter, Copy, Clone)]
pub enum RateLimitType {
  Message,
  Register,
  Post,
}

/// Rate limiting based on rate type and IP addr
#[derive(Debug, Clone)]
pub struct RateLimiter {
  pub buckets: HashMap<RateLimitType, HashMap<IPAddr, RateLimitBucket>>,
}

impl Default for RateLimiter {
  fn default() -> Self {
    Self {
      buckets: HashMap::new(),
    }
  }
}

impl RateLimiter {
  fn insert_ip(&mut self, ip: &str) {
    for rate_limit_type in RateLimitType::iter() {
      if self.buckets.get(&rate_limit_type).is_none() {
        self.buckets.insert(rate_limit_type, HashMap::new());
      }

      if let Some(bucket) = self.buckets.get_mut(&rate_limit_type) {
        if bucket.get(ip).is_none() {
          bucket.insert(
            ip.to_string(),
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
    ip: &str,
    rate: i32,
    per: i32,
    check_only: bool,
  ) -> Result<(), Error> {
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
            "Rate limited IP: {}, time_passed: {}, allowance: {}",
            ip, time_passed, rate_limit.allowance
          );
          Err(
            APIError {
              message: format!("Too many requests. {} per {} seconds", rate, per),
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
