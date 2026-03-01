use crate::rate_limit::{
  backend::LemmyBackend,
  input::{LemmyInput, LemmyInputFuture, raw_ip_key},
};
use actix_extensible_rate_limit::{RateLimiter, backend::SimpleOutput};
use actix_web::dev::ServiceRequest;
use enum_map::{EnumMap, enum_map};
use std::future::ready;
use strum::{AsRefStr, Display};

mod backend;
mod input;

#[derive(Debug, enum_map::Enum, Copy, Clone, Display, AsRefStr, Eq, PartialEq, Hash)]
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
  backend: LemmyBackend,
}

impl RateLimit {
  pub fn new(configs: EnumMap<ActionType, BucketConfig>) -> Self {
    Self {
      backend: LemmyBackend::new(configs, true),
    }
  }

  pub fn with_debug_config() -> Self {
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

  #[expect(clippy::expect_used)]
  pub fn set_config(&self, configs: EnumMap<ActionType, BucketConfig>) {
    *self.backend.configs.write().expect("write rwlock") = configs;
  }

  fn build_rate_limiter(
    &self,
    action_type: ActionType,
  ) -> RateLimiter<LemmyBackend, SimpleOutput, impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static>
  {
    let input = new_input(action_type);

    RateLimiter::builder(self.backend.clone(), input)
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

fn new_input(action_type: ActionType) -> impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static {
  move |req| {
    ready({
      let info = req.connection_info();
      let key = raw_ip_key(info.realip_remote_addr());

      Ok(LemmyInput(key, action_type))
    })
  }
}
