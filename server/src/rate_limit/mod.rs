pub mod rate_limiter;

use super::{IPAddr, Settings};
use crate::api::APIError;
use crate::settings::RateLimitConfig;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use failure::Error;
use futures::future::{ok, Ready};
use log::warn;
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
pub struct RateLimit(pub Arc<Mutex<RateLimiter>>);

#[derive(Debug, Clone)]
pub struct RateLimited(Arc<Mutex<RateLimiter>>, RateLimitType);

pub struct RateLimitedMiddleware<S>(RateLimited, S);

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
    RateLimited(self.0.clone(), type_)
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
      Ok(Settings::get().rate_limit.clone()) as Result<_, failure::Error>
    })
    .await
    .map_err(|e| match e {
      actix_web::error::BlockingError::Error(e) => e,
      _ => APIError::err("Operation canceled").into(),
    })?;

    // before
    {
      let mut limiter = self.0.lock().await;

      match self.1 {
        RateLimitType::Message => {
          limiter.check_rate_limit_full(
            self.1,
            &ip_addr,
            rate_limit.message,
            rate_limit.message_per_second,
            false,
          )?;

          return fut.await;
        }
        RateLimitType::Post => {
          limiter.check_rate_limit_full(
            self.1.clone(),
            &ip_addr,
            rate_limit.post,
            rate_limit.post_per_second,
            true,
          )?;
        }
        RateLimitType::Register => {
          limiter.check_rate_limit_full(
            self.1,
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
      let mut limiter = self.0.lock().await;
      if res.is_ok() {
        match self.1 {
          RateLimitType::Post => {
            limiter.check_rate_limit_full(
              self.1,
              &ip_addr,
              rate_limit.post,
              rate_limit.post_per_second,
              false,
            )?;
          }
          RateLimitType::Register => {
            limiter.check_rate_limit_full(
              self.1,
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
    ok(RateLimitedMiddleware(self.clone(), service))
  }
}

impl<S> Service for RateLimitedMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
  S::Future: 'static,
{
  type Request = S::Request;
  type Response = S::Response;
  type Error = actix_web::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.1.poll_ready(cx)
  }

  fn call(&mut self, req: S::Request) -> Self::Future {
    let ip_addr = req
      .connection_info()
      .remote()
      .unwrap_or("127.0.0.1:12345")
      .split(':')
      .next()
      .unwrap_or("127.0.0.1")
      .to_string();

    let fut = self.0.clone().wrap(ip_addr, self.1.call(req));

    Box::pin(async move { fut.await.map_err(actix_web::Error::from) })
  }
}
