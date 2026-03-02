//! The content in this file is mostly copy-pasted from library code:
//! https://github.com/jacob-pro/actix-extensible-rate-limit/blob/master/src/backend/memory.rs

use crate::rate_limit::{ActionType, BucketConfig, input::LemmyInput};
use actix_extensible_rate_limit::backend::{
  Backend,
  Decision,
  SimpleOutput,
  memory::DEFAULT_GC_INTERVAL_SECONDS,
};
use actix_web::rt::{task::JoinHandle, time::Instant};
use dashmap::DashMap;
use enum_map::EnumMap;
use std::{
  convert::Infallible,
  sync::{Arc, RwLock},
  time::Duration,
};

/// A Fixed Window rate limiter [Backend] that uses [Dashmap](dashmap::DashMap) to store keys
/// in memory.
#[derive(Clone)]
pub struct LemmyBackend {
  map: Arc<DashMap<LemmyInput, Value>>,
  gc_handle: Option<Arc<JoinHandle<()>>>,
  pub(super) configs: Arc<RwLock<EnumMap<ActionType, BucketConfig>>>,
}

struct Value {
  ttl: Instant,
  count: u64,
}

impl LemmyBackend {
  pub(crate) fn new(configs: EnumMap<ActionType, BucketConfig>, enable_gc: bool) -> Self {
    let map = Arc::new(DashMap::<LemmyInput, Value>::new());
    let gc_handle = enable_gc.then(|| {
      Arc::new(LemmyBackend::garbage_collector(
        map.clone(),
        Duration::from_secs(DEFAULT_GC_INTERVAL_SECONDS),
      ))
    });
    LemmyBackend {
      map,
      gc_handle,
      configs: Arc::new(RwLock::new(configs)),
    }
  }

  fn garbage_collector(map: Arc<DashMap<LemmyInput, Value>>, interval: Duration) -> JoinHandle<()> {
    assert!(
      interval.as_secs_f64() > 0f64,
      "GC interval must be non-zero"
    );
    tokio::spawn(async move {
      loop {
        let now = Instant::now();
        map.retain(|_k, v| v.ttl > now);
        tokio::time::sleep_until(now + interval).await;
      }
    })
  }
}

impl Backend<LemmyInput> for LemmyBackend {
  type Output = SimpleOutput;
  type RollbackToken = LemmyInput;
  type Error = Infallible;

  #[expect(clippy::expect_used)]
  async fn request(
    &self,
    input: LemmyInput,
  ) -> Result<(Decision, Self::Output, Self::RollbackToken), Self::Error> {
    #[expect(clippy::expect_used)]
    let config = self.configs.read().expect("read rwlock")[input.1];

    let max_requests: u64 = config.max_requests.into();
    let interval = Duration::from_secs(config.interval.into());

    let now = Instant::now();
    let mut count = 1;
    let mut expiry = now
      .checked_add(interval)
      .expect("Interval unexpectedly large");
    self
      .map
      .entry(input)
      .and_modify(|v| {
        // If this bucket hasn't yet expired, increment and extract the count/expiry
        if v.ttl > now {
          v.count += 1;
          count = v.count;
          expiry = v.ttl;
        } else {
          // If this bucket has expired we will reset the count to 1 and set a new TTL.
          v.ttl = expiry;
          v.count = count;
        }
      })
      .or_insert_with(|| Value {
        // If the bucket doesn't exist, create it with a count of 1, and set the TTL.
        ttl: expiry,
        count,
      });
    let allow = count <= max_requests;
    let output = SimpleOutput {
      limit: max_requests,
      remaining: max_requests.saturating_sub(count),
      reset: expiry,
    };
    Ok((Decision::from_allowed(allow), output, input))
  }

  async fn rollback(&self, token: Self::RollbackToken) -> Result<(), Self::Error> {
    self.map.entry(token).and_modify(|v| {
      v.count = v.count.saturating_sub(1);
    });
    Ok(())
  }
}

impl Drop for LemmyBackend {
  fn drop(&mut self) {
    if let Some(handle) = &self.gc_handle {
      handle.abort();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    error::LemmyResult,
    rate_limit::{ActionType, input::raw_ip_key},
  };
  use enum_map::enum_map;

  const MINUTE_SECS: u32 = 60;
  const MINUTE: Duration = Duration::from_secs(60);

  fn test_config(interval: u32, max_requests: u32) -> EnumMap<ActionType, BucketConfig> {
    enum_map! {
        ActionType::Message => BucketConfig {
          max_requests,
          interval
        },
        ActionType::Post => BucketConfig {
          max_requests: 1,
          interval: 120,
        },
        ActionType::Register => BucketConfig {
          max_requests: 0,
          interval: 0,
        },
        ActionType::Image => BucketConfig {
          max_requests: 0,
          interval: 0,
        },
        ActionType::Comment => BucketConfig {
          max_requests: 0,
          interval: 0,
        },
        ActionType::Search => BucketConfig {
          max_requests: 0,
          interval: 0,
        },
        ActionType::ImportUserSettings => BucketConfig {
          max_requests: 0,
          interval: 0,
        },
    }
  }

  #[actix_web::test]
  async fn test_allow_deny() -> LemmyResult<()> {
    tokio::time::pause();
    let backend = LemmyBackend::new(test_config(MINUTE_SECS, 5), true);
    let key = raw_ip_key(Some("127.0.0.2"));
    let input = LemmyInput(key, ActionType::Message);
    for _ in 0..5 {
      // First 5 should be allowed
      let (allow, _, _) = backend.request(input).await?;
      assert!(allow.is_allowed());
    }
    // Sixth should be denied
    let (allow, _, _) = backend.request(input).await?;
    assert!(!allow.is_allowed());
    Ok(())
  }

  #[actix_web::test]
  async fn test_reset() -> LemmyResult<()> {
    tokio::time::pause();
    let backend = LemmyBackend::new(test_config(MINUTE_SECS, 1), false);
    let input = LemmyInput(raw_ip_key(Some("127.0.0.3")), ActionType::Message);
    // Make first request, should be allowed
    let (decision, _, _) = backend.request(input).await?;
    assert!(decision.is_allowed());
    // Request again, should be denied
    let (decision, _, _) = backend.request(input).await?;
    assert!(decision.is_denied());
    // Advance time and try again, should now be allowed
    tokio::time::advance(MINUTE).await;
    // We want to be sure the key hasn't been garbage collected, and we are testing the expiry logic
    assert!(backend.map.contains_key(&input));
    let (decision, _, _) = backend.request(input).await?;
    assert!(decision.is_allowed());
    Ok(())
  }

  #[actix_web::test]
  async fn test_garbage_collection() -> LemmyResult<()> {
    tokio::time::pause();
    let backend = LemmyBackend::new(test_config(MINUTE_SECS, 1), true);
    let key1 = LemmyInput(raw_ip_key(Some("127.0.0.4")), ActionType::Message);
    let key2 = LemmyInput(raw_ip_key(Some("127.0.0.5")), ActionType::Post);
    backend.request(key1).await?;
    backend.request(key2).await?;
    assert!(backend.map.contains_key(&key1));
    assert!(backend.map.contains_key(&key2));
    // Advance time such that the garbage collector runs,
    // expired KEY1 should be cleaned, but KEY2 should remain.
    tokio::time::advance(MINUTE).await;
    assert!(!backend.map.contains_key(&key1));
    assert!(backend.map.contains_key(&key2));
    Ok(())
  }

  #[actix_web::test]
  async fn test_output() -> LemmyResult<()> {
    tokio::time::pause();
    let backend = LemmyBackend::new(test_config(MINUTE_SECS, 2), true);
    let key = raw_ip_key(Some("127.0.0.6"));
    let input = LemmyInput(key, ActionType::Message);
    // First of 2 should be allowed.
    let (decision, output, _) = backend.request(input).await?;
    assert!(decision.is_allowed());
    assert_eq!(output.remaining, 1);
    assert_eq!(output.limit, 2);
    assert_eq!(output.reset, Instant::now() + MINUTE);
    // Second of 2 should be allowed.
    let (decision, output, _) = backend.request(input).await?;
    assert!(decision.is_allowed());
    assert_eq!(output.remaining, 0);
    assert_eq!(output.limit, 2);
    assert_eq!(output.reset, Instant::now() + MINUTE);
    // Should be denied
    let (decision, output, _) = backend.request(input).await?;
    assert!(decision.is_denied());
    assert_eq!(output.remaining, 0);
    assert_eq!(output.limit, 2);
    assert_eq!(output.reset, Instant::now() + MINUTE);
    Ok(())
  }

  #[actix_web::test]
  async fn test_rollback() -> LemmyResult<()> {
    tokio::time::pause();
    let backend = LemmyBackend::new(test_config(MINUTE_SECS, 5), true);
    let key = raw_ip_key(Some("127.0.0.7"));
    let input = LemmyInput(key, ActionType::Message);
    let (_, output, rollback) = backend.request(input).await?;
    assert_eq!(output.remaining, 4);
    backend.rollback(rollback).await?;
    // Remaining requests should still be the same, since the previous call was excluded
    let (_, output, _) = backend.request(input).await?;
    assert_eq!(output.remaining, 4);
    Ok(())
  }
}
