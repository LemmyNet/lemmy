use actix_extensible_rate_limit::{
  backend::{
    memory::InMemoryBackend,
    raw_ip_key,
    MyIpAddr,
    SimpleInput,
    SimpleInputFuture,
    SimpleOutput,
  },
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
  backends: EnumMap<ActionType, InMemoryBackend<MyIpAddr>>,
}

impl RateLimit {
  pub fn new(configs: EnumMap<ActionType, BucketConfig>) -> Self {
    Self {
      configs: Arc::new(RwLock::new(configs)),
      backends: EnumMap::from_fn(|_| InMemoryBackend::<MyIpAddr>::builder().build()),
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

  #[allow(clippy::expect_used)]
  pub fn set_config(&self, configs: EnumMap<ActionType, BucketConfig>) {
    *self.configs.write().expect("write rwlock") = configs;
  }

  fn build_rate_limiter(
    &self,
    action_type: ActionType,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    let input = new_input(action_type, self.configs.clone());

    RateLimiter::builder(self.backends[action_type].clone(), input)
      .add_headers()
      // rollback rate limit on any error 500
      .rollback_server_errors()
      .build()
  }

  pub fn message(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::Message)
  }

  pub fn search(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::Search)
  }
  pub fn register(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::Register)
  }
  pub fn post(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::Post)
  }
  pub fn image(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::Image)
  }
  pub fn comment(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::Comment)
  }
  pub fn import_user_settings(
    &self,
  ) -> RateLimiter<
    InMemoryBackend<MyIpAddr>,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static,
  > {
    self.build_rate_limiter(ActionType::ImportUserSettings)
  }
}

fn new_input(
  action_type: ActionType,
  configs: Arc<RwLock<EnumMap<ActionType, BucketConfig>>>,
) -> impl Fn(&ServiceRequest) -> SimpleInputFuture<MyIpAddr> + 'static {
  move |req| {
    ready({
      let info = req.connection_info();
      let key = raw_ip_key(info.realip_remote_addr());

      #[allow(clippy::expect_used)]
      let config = configs.read().expect("read rwlock")[action_type];

      let interval = Duration::from_secs(config.secs_to_refill.into());
      let max_requests = config.capacity.into();
      Ok(SimpleInput {
        interval,
        max_requests,
        key,
      })
    })
  }
}
