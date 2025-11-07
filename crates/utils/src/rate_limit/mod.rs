use crate::rate_limit::{
  backend::LemmyBackend,
  input::{LemmyInput, LemmyInputFuture, raw_ip_key},
};
use actix_extensible_rate_limit::{RateLimiter, backend::SimpleOutput};
use actix_web::dev::ServiceRequest;
use enum_map::{EnumMap, enum_map};
use std::{
  future::ready,
  sync::{Arc, RwLock},
  time::Duration,
};
use strum::{AsRefStr, Display};

mod backend;
mod input;

#[derive(Debug, enum_map::Enum, Copy, Clone, Display, AsRefStr)]
pub enum ActionType {
  Message,
  Register,
  Post,
  Image,
  Comment,
  Search,
  ImportUserSettings,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct BucketConfig {
  pub max_requests: u32,
  pub interval: u32,
}

#[derive(Clone)]
pub struct RateLimit {
  configs: Arc<RwLock<EnumMap<ActionType, BucketConfig>>>,
  backends: EnumMap<ActionType, LemmyBackend>,
}

impl RateLimit {
  pub fn new(configs: EnumMap<ActionType, BucketConfig>) -> Self {
    Self {
      configs: Arc::new(RwLock::new(configs)),
      backends: EnumMap::from_fn(|_| LemmyBackend::default()),
    }
  }

  pub fn with_test_config() -> Self {
    Self::new(enum_map! {
      ActionType::Message => BucketConfig {
        max_requests: 180,
        interval: 60,
      },
      ActionType::Post => BucketConfig {
        max_requests: 6,
        interval: 300,
      },
      ActionType::Register => BucketConfig {
        max_requests: 3,
        interval: 3600,
      },
      ActionType::Image => BucketConfig {
        max_requests: 6,
        interval: 3600,
      },
      ActionType::Comment => BucketConfig {
        max_requests: 6,
        interval: 600,
      },
      ActionType::Search => BucketConfig {
        max_requests: 60,
        interval: 600,
      },
      ActionType::ImportUserSettings => BucketConfig {
        max_requests: 1,
        interval: 24 * 60 * 60,
      },
    })
  }

  #[allow(clippy::expect_used)]
  pub fn set_config(&self, configs: EnumMap<ActionType, BucketConfig>) {
    *self.configs.write().expect("write rwlock") = configs;
  }

  fn build_rate_limiter(
    &self,
    action_type: ActionType,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    let input = new_input(action_type, self.configs.clone());

    RateLimiter::builder(self.backends[action_type].clone(), input)
      .add_headers()
      // rollback rate limit on any error 500
      .rollback_server_errors()
      .build()
  }

  pub fn message(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::Message)
  }

  pub fn search(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::Search)
  }
  pub fn register(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::Register)
  }
  pub fn post(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::Post)
  }
  pub fn image(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::Image)
  }
  pub fn comment(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::Comment)
  }
  pub fn import_user_settings(
    &self,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    self.build_rate_limiter(ActionType::ImportUserSettings)
  }
}

fn new_input(
  action_type: ActionType,
  configs: Arc<RwLock<EnumMap<ActionType, BucketConfig>>>,
) -> impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static {
  move |req| {
    ready({
      let info = req.connection_info();
      let key = raw_ip_key(info.realip_remote_addr());

      #[allow(clippy::expect_used)]
      let config = configs.read().expect("read rwlock")[action_type];

      let interval = Duration::from_secs(config.interval.into());
      let max_requests = config.max_requests.into();
      Ok(LemmyInput {
        interval,
        max_requests,
        key,
      })
    })
  }
}
