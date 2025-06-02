use actix_extensible_rate_limit::{
  backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder, SimpleInputFuture, SimpleOutput},
  RateLimiter,
};
use actix_web::dev::ServiceRequest;
use lemmy_db_schema::source::local_site_rate_limit::LocalSiteRateLimit;
use std::time::Duration;

pub struct RateLimit {
  limits: LocalSiteRateLimit,
  message: InMemoryBackend,
}

impl RateLimit {
  pub fn new(limits: LocalSiteRateLimit) -> Self {
    Self {
      limits,
      message: InMemoryBackend::builder().build(),
    }
  }

  fn build_rate_limiter(
    backend: &InMemoryBackend,
    interval: Duration,
    mut max_requests: u64,
  ) -> RateLimiter<
    InMemoryBackend,
    SimpleOutput,
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    // TODO these have to be set, because the database defaults are too low for the federation
    // tests to pass, and there's no way to live update the rate limits without restarting the
    // server.
    // This can be removed once live rate limits are enabled.
    if cfg!(debug_assertions) {
      max_requests = 999;
    }
    let input = SimpleInputFunctionBuilder::new(interval, max_requests)
      .real_ip_key()
      .build();
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
    impl Fn(&ServiceRequest) -> SimpleInputFuture + 'static,
  > {
    Self::build_rate_limiter(
      &self.message,
      Duration::from_secs(self.limits.message.try_into().unwrap_or(0)),
      // TODO: rename db fields as message_per_interval
      self.limits.message_per_second.try_into().unwrap_or(0),
    )
  }
}
