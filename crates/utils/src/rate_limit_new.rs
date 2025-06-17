use crate::rate_limit::{ActionType, BucketConfig};
use actix_extensible_rate_limit::{
  backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder, SimpleInputFuture, SimpleOutput},
  RateLimiter,
};
use actix_web::dev::ServiceRequest;
use enum_map::EnumMap;
use std::time::Duration;

pub struct RateLimit {
  configs: EnumMap<ActionType, BucketConfig>,
  backends: EnumMap<ActionType, InMemoryBackend>,
}

impl RateLimit {
  pub fn new(configs: EnumMap<ActionType, BucketConfig>) -> Self {
    Self {
      configs,
      backends: EnumMap::from_fn(|_| InMemoryBackend::builder().build()),
    }
  }

  fn build_rate_limiter(
    &self,
    action_type: ActionType,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    let mut config = self.configs[action_type];
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
    let input = SimpleInputFunctionBuilder::new(interval, max_requests)
      .real_ip_key()
      .build();

    RateLimiter::builder(self.backends[action_type].clone(), input)
      .add_headers()
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
