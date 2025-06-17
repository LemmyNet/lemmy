use actix_extensible_rate_limit::{
  backend::{ip_key, memory::InMemoryBackend, SimpleInput, SimpleInputFuture, SimpleOutput},
  RateLimiter,
};
use actix_web::dev::ServiceRequest;
use enum_map::{enum_map, EnumMap};
use std::{
  future::ready,
  sync::{Arc, RwLock},
  time::Duration,
};
use strum::{AsRefStr, Display};

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
  pub capacity: u32,
  pub secs_to_refill: u32,
}

#[derive(Clone)]
pub struct RateLimit {
  configs: Arc<RwLock<EnumMap<ActionType, BucketConfig>>>,
  backends: EnumMap<ActionType, InMemoryBackend>,
}

impl RateLimit {
  pub fn new(configs: EnumMap<ActionType, BucketConfig>) -> Self {
    Self {
      configs: Arc::new(RwLock::new(configs)),
      backends: EnumMap::from_fn(|_| InMemoryBackend::builder().build()),
    }
  }

  pub fn with_test_config() -> Self {
    Self::new(enum_map! {
      ActionType::Message => BucketConfig {
        capacity: 180,
        secs_to_refill: 60,
      },
      ActionType::Post => BucketConfig {
        capacity: 6,
        secs_to_refill: 300,
      },
      ActionType::Register => BucketConfig {
        capacity: 3,
        secs_to_refill: 3600,
      },
      ActionType::Image => BucketConfig {
        capacity: 6,
        secs_to_refill: 3600,
      },
      ActionType::Comment => BucketConfig {
        capacity: 6,
        secs_to_refill: 600,
      },
      ActionType::Search => BucketConfig {
        capacity: 60,
        secs_to_refill: 600,
      },
      ActionType::ImportUserSettings => BucketConfig {
        capacity: 1,
        secs_to_refill: 24 * 60 * 60,
      },
    })
  }

  pub fn set_config(&self, configs: EnumMap<ActionType, BucketConfig>) {
    *self.configs.write().unwrap() = configs;
  }

  fn build_rate_limiter(
    &self,
    action_type: ActionType,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    let input = new_input(action_type, self.configs.clone());

    RateLimiter::builder(self.backends[action_type].clone(), input)
      .add_headers()
      // TODO: should only rollback on specific errors eg wrong captcha
      .rollback_server_errors()
      .build()
  }

  pub fn message(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Message)
  }

  pub fn search(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Search)
  }
  pub fn register(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Register)
  }
  pub fn post(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Post)
  }
  pub fn image(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Image)
  }
  pub fn comment(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Comment)
  }
  pub fn import_user_settings(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::ImportUserSettings)
  }
}

/// https://github.com/jacob-pro/actix-extensible-rate-limit/blob/master/src/backend/input_builder.rs#L92
fn new_input(
  action_type: ActionType,
  configs: Arc<RwLock<EnumMap<ActionType, BucketConfig>>>,
) -> impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static {
  move |req| {
    ready((|| {
      let info = req.connection_info();
      let key = ip_key(info.realip_remote_addr().unwrap())?;

      let mut config = configs.read().unwrap()[action_type];
      // TODO these have to be set, because the database defaults are too low for the federation
      // tests to pass, and there's no way to live update the rate limits without restarting the
      // server.
      // This can be removed once live rate limits are enabled.
      if cfg!(debug_assertions) {
        config.capacity = 999;
      }

      // TODO: rename rust and db fields to be consistent
      let interval = Duration::from_secs(config.secs_to_refill.try_into().unwrap_or(0));
      let max_requests = config.capacity.try_into().unwrap_or(0);

      Ok(SimpleInput {
        interval,
        max_requests,
        key,
      })
    })())
  }
}
