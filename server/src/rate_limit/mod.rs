pub mod rate_limiter;

use super::{IPAddr, Settings};
use crate::api::APIError;
use crate::get_ip;
use crate::settings::RateLimitConfig;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use failure::Error;
use futures::future::{ok, Ready};
use log::debug;
use rate_limiter::{RateLimitType, RateLimiter};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::SystemTime;
use strum::IntoEnumIterator;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimit {
  pub rate_limiter: Arc<Mutex<RateLimiter>>,
}

#[derive(Debug, Clone)]
pub struct RateLimited {
  rate_limiter: Arc<Mutex<RateLimiter>>,
  type_: RateLimitType,
}

pub struct RateLimitedMiddleware<S> {
  rate_limited: RateLimited,
  service: S,
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

  fn kind(&self, type_: RateLimitType) -> RateLimited {
    RateLimited {
      rate_limiter: self.rate_limiter.clone(),
      type_,
    }
  }
}

impl RateLimited {
  pub async fn wrap<T, E>(
    self,
    ip_addr: String,
    fut: impl Future<Output = Result<T, E>>,
  ) -> Result<T, E>
  where
    E: From<failure::Error>,
  {
    let rate_limit: RateLimitConfig = actix_web::web::block(move || {
      // needs to be in a web::block because the RwLock in settings is from stdlib
      Ok(Settings::get().rate_limit) as Result<_, failure::Error>
    })
    .await
    .map_err(|e| match e {
      actix_web::error::BlockingError::Error(e) => e,
      _ => APIError::err("Operation canceled").into(),
    })?;

    // before
    {
      let mut limiter = self.rate_limiter.lock().await;

      match self.type_ {
        RateLimitType::Message => {
          limiter.check_rate_limit_full(
            self.type_,
            &ip_addr,
            rate_limit.message,
            rate_limit.message_per_second,
            false,
          )?;

          return fut.await;
        }
        RateLimitType::Post => {
          limiter.check_rate_limit_full(
            self.type_.clone(),
            &ip_addr,
            rate_limit.post,
            rate_limit.post_per_second,
            true,
          )?;
        }
        RateLimitType::Register => {
          limiter.check_rate_limit_full(
            self.type_,
            &ip_addr,
            rate_limit.register,
            rate_limit.register_per_second,
            true,
          )?;
        }
      };
    }

    let res = fut.await;

    // after
    {
      let mut limiter = self.rate_limiter.lock().await;
      if res.is_ok() {
        match self.type_ {
          RateLimitType::Post => {
            limiter.check_rate_limit_full(
              self.type_,
              &ip_addr,
              rate_limit.post,
              rate_limit.post_per_second,
              false,
            )?;
          }
          RateLimitType::Register => {
            limiter.check_rate_limit_full(
              self.type_,
              &ip_addr,
              rate_limit.register,
              rate_limit.register_per_second,
              false,
            )?;
          }
          _ => (),
        };
      }
    }

    res
  }
}

impl<S> Transform<S> for RateLimited
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
  S::Future: 'static,
{
  type Request = S::Request;
  type Response = S::Response;
  type Error = actix_web::Error;
  type InitError = ();
  type Transform = RateLimitedMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(RateLimitedMiddleware {
      rate_limited: self.clone(),
      service,
    })
  }
}

type FutResult<T, E> = dyn Future<Output = Result<T, E>>;

impl<S> Service for RateLimitedMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
  S::Future: 'static,
{
  type Request = S::Request;
  type Response = S::Response;
  type Error = actix_web::Error;
  type Future = Pin<Box<FutResult<Self::Response, Self::Error>>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, req: S::Request) -> Self::Future {
    let ip_addr = get_ip(&req.connection_info());

    let fut = self
      .rate_limited
      .clone()
      .wrap(ip_addr, self.service.call(req));

    Box::pin(async move { fut.await.map_err(actix_web::Error::from) })
  }
}
