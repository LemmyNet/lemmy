use crate::{settings::structs::RateLimitConfig, utils::get_ip, IpAddr};
use actix_web::{
  dev::{Service, ServiceRequest, ServiceResponse, Transform},
  HttpResponse,
};
use futures::future::{ok, Ready};
use rate_limiter::{RateLimitType, RateLimiter};
use std::{
  future::Future,
  pin::Pin,
  rc::Rc,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};

pub mod rate_limiter;

#[derive(Debug, Clone)]
pub struct RateLimit {
  // it might be reasonable to use a std::sync::Mutex here, since we don't need to lock this
  // across await points
  pub rate_limiter: Arc<Mutex<RateLimiter>>,
  pub rate_limit_config: RateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct RateLimited {
  rate_limiter: Arc<Mutex<RateLimiter>>,
  rate_limit_config: RateLimitConfig,
  type_: RateLimitType,
}

pub struct RateLimitedMiddleware<S> {
  rate_limited: RateLimited,
  service: Rc<S>,
}

impl RateLimit {
  pub fn message(&self) -> RateLimited {
    self.kind(RateLimitType::Message)
  }

  pub fn post(&self) -> RateLimited {
    self.kind(RateLimitType::Post)
  }

  pub fn register(&self) -> RateLimited {
    self.kind(RateLimitType::Register)
  }

  pub fn image(&self) -> RateLimited {
    self.kind(RateLimitType::Image)
  }

  pub fn comment(&self) -> RateLimited {
    self.kind(RateLimitType::Comment)
  }

  pub fn search(&self) -> RateLimited {
    self.kind(RateLimitType::Search)
  }

  fn kind(&self, type_: RateLimitType) -> RateLimited {
    RateLimited {
      rate_limiter: self.rate_limiter.clone(),
      rate_limit_config: self.rate_limit_config.clone(),
      type_,
    }
  }
}

impl RateLimited {
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub fn check(self, ip_addr: IpAddr) -> bool {
    // Does not need to be blocking because the RwLock in settings never held across await points,
    // and the operation here locks only long enough to clone
    let rate_limit = self.rate_limit_config;

    let (kind, interval) = match self.type_ {
      RateLimitType::Message => (rate_limit.message, rate_limit.message_per_second),
      RateLimitType::Post => (rate_limit.post, rate_limit.post_per_second),
      RateLimitType::Register => (rate_limit.register, rate_limit.register_per_second),
      RateLimitType::Image => (rate_limit.image, rate_limit.image_per_second),
      RateLimitType::Comment => (rate_limit.comment, rate_limit.comment_per_second),
      RateLimitType::Search => (rate_limit.search, rate_limit.search_per_second),
    };
    let mut limiter = self.rate_limiter.lock().expect("mutex poison error");

    limiter.check_rate_limit_full(self.type_, &ip_addr, kind, interval)
  }
}

impl<S> Transform<S, ServiceRequest> for RateLimited
where
  S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
  S::Future: 'static,
{
  type Response = S::Response;
  type Error = actix_web::Error;
  type InitError = ();
  type Transform = RateLimitedMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(RateLimitedMiddleware {
      rate_limited: self.clone(),
      service: Rc::new(service),
    })
  }
}

type FutResult<T, E> = dyn Future<Output = Result<T, E>>;

impl<S> Service<ServiceRequest> for RateLimitedMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
  S::Future: 'static,
{
  type Response = S::Response;
  type Error = actix_web::Error;
  type Future = Pin<Box<FutResult<Self::Response, Self::Error>>>;

  fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let ip_addr = get_ip(&req.connection_info());

    let rate_limited = self.rate_limited.clone();
    let service = self.service.clone();

    Box::pin(async move {
      if rate_limited.check(ip_addr) {
        service.call(req).await
      } else {
        let (http_req, _) = req.into_parts();
        // if rate limit was hit, respond with http 400
        Ok(ServiceResponse::new(
          http_req,
          HttpResponse::BadRequest().finish(),
        ))
      }
    })
  }
}
