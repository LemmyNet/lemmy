use actix_extensible_rate_limit::{
  backend::{memory::InMemoryBackend, Backend, Decision, SimpleOutput},
  RateLimiter,
};
use actix_web::dev::ServiceRequest;
use enum_map::EnumMap;
use lemmy_utils::rate_limit::{ActionType, BucketConfig};
use std::{
  convert::Infallible,
  future::{ready, Ready},
  sync::{Arc, Mutex},
};

type RateLimitConfig = Arc<Mutex<EnumMap<ActionType, BucketConfig>>>;

pub struct RateLimit {
  limits: RateLimitConfig,
  message: InMemoryBackend,
}

pub struct LemmyInput {
  limits: RateLimitConfig,
}

pub type LemmyInputFuture = Ready<Result<LemmyInput, actix_web::Error>>;

impl RateLimit {
  pub fn new(limits: RateLimitConfig) -> Self {
    Self {
      limits,
      message: InMemoryBackend::builder().build(),
    }
  }

  fn build_rate_limiter(
    &self,
    action_type: ActionType,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static,
  > {
    // TODO these have to be set, because the database defaults are too low for the federation
    // tests to pass, and there's no way to live update the rate limits without restarting the
    // server.
    // This can be removed once live rate limits are enabled.
    if cfg!(debug_assertions) {
      max_requests = 999;
    }

    let input = move |req: &ServiceRequest| {
      ready((|| {
        let mut components: Vec<String> = Vec::new();
        let info = req.connection_info();
        // https://github.com/jacob-pro/actix-extensible-rate-limit/blob/master/src/backend/input_builder.rs#L139
        components.push(todo!());

        let key = components.join("-");

        Ok(LemmyInput {
          limits: self.limits,
        })
      })())
    };
    // TODO: use different backend depending on action_type
    let backend = self.message;
    RateLimiter::builder(backend.clone(), input)
      .add_headers()
      .rollback_server_errors()
      .build()
  }

  pub fn message(
    &self,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> LemmyInputFuture + 'static,
  > {
    self.build_rate_limiter(ActionType::Message)
  }
}

// https://github.com/jacob-pro/actix-extensible-rate-limit/blob/master/src/backend/memory.rs#L70
impl Backend<LemmyInput> for InMemoryBackend {
  type Output = SimpleOutput;

  type RollbackToken = String;

  type Error = Infallible;

  async fn request(
    &self,
    input: LemmyInput,
  ) -> Result<(Decision, Self::Output, Self::RollbackToken), Self::Error> {
    todo!()
  }

  async fn rollback(&self, token: Self::RollbackToken) -> Result<(), Self::Error> {
    self.map.entry(token).and_modify(|v| {
      v.count = v.count.saturating_sub(1);
    });
    Ok(())
  }
}
