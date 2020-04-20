pub mod rate_limiter;

use super::{IPAddr, Settings};
use crate::api::APIError;
use failure::Error;
use log::debug;
use rate_limiter::RateLimiter;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
  pub rate_limiter: Arc<Mutex<RateLimiter>>,
  pub ip: IPAddr,
}
