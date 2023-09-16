use crate::error::{LemmyError, LemmyErrorType};
use actix_web::dev::{ConnectionInfo, Service, ServiceRequest, ServiceResponse, Transform};
use enum_map::{enum_map, EnumMap};
use futures::future::{ok, Ready};
use rate_limiter::{BucketConfig, InstantSecs, RateLimitStorage, RateLimitType};
use serde::{Deserialize, Serialize};
use std::{
  future::Future,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  pin::Pin,
  rc::Rc,
  str::FromStr,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};
use tokio::sync::{mpsc, mpsc::Sender, OnceCell};
use typed_builder::TypedBuilder;

pub mod rate_limiter;

#[derive(Debug, Deserialize, Serialize, Clone, TypedBuilder)]
pub struct RateLimitConfig {
  #[builder(default = 180)]
  /// Maximum number of messages created in interval
  pub message: i32,
  #[builder(default = 60)]
  /// Interval length for message limit, in seconds
  pub message_per_second: i32,
  #[builder(default = 6)]
  /// Maximum number of posts created in interval
  pub post: i32,
  #[builder(default = 300)]
  /// Interval length for post limit, in seconds
  pub post_per_second: i32,
  #[builder(default = 3)]
  /// Maximum number of registrations in interval
  pub register: i32,
  #[builder(default = 3600)]
  /// Interval length for registration limit, in seconds
  pub register_per_second: i32,
  #[builder(default = 6)]
  /// Maximum number of image uploads in interval
  pub image: i32,
  #[builder(default = 3600)]
  /// Interval length for image uploads, in seconds
  pub image_per_second: i32,
  #[builder(default = 6)]
  /// Maximum number of comments created in interval
  pub comment: i32,
  #[builder(default = 600)]
  /// Interval length for comment limit, in seconds
  pub comment_per_second: i32,
  #[builder(default = 60)]
  /// Maximum number of searches created in interval
  pub search: i32,
  #[builder(default = 600)]
  /// Interval length for search limit, in seconds
  pub search_per_second: i32,
}

impl From<RateLimitConfig> for EnumMap<RateLimitType, BucketConfig> {
  fn from(rate_limit: RateLimitConfig) -> Self {
    enum_map! {
      RateLimitType::Message => (rate_limit.message, rate_limit.message_per_second),
      RateLimitType::Post => (rate_limit.post, rate_limit.post_per_second),
      RateLimitType::Register => (rate_limit.register, rate_limit.register_per_second),
      RateLimitType::Image => (rate_limit.image, rate_limit.image_per_second),
      RateLimitType::Comment => (rate_limit.comment, rate_limit.comment_per_second),
      RateLimitType::Search => (rate_limit.search, rate_limit.search_per_second),
    }
    .map(|_, t| BucketConfig {
      capacity: t.0,
      secs_to_refill: t.1,
    })
  }
}

#[derive(Debug, Clone)]
struct RateLimit {
  pub rate_limiter: RateLimitStorage,
}

#[derive(Debug, Clone)]
pub struct RateLimitedGuard {
  rate_limit: Arc<Mutex<RateLimit>>,
  type_: RateLimitType,
}

/// Single instance of rate limit config and buckets, which is shared across all threads.
#[derive(Clone)]
pub struct RateLimitCell {
  tx: Sender<RateLimitConfig>,
  rate_limit: Arc<Mutex<RateLimit>>,
}

impl RateLimitCell {
  /// Initialize cell if it wasnt initialized yet. Otherwise returns the existing cell.
  pub async fn new(rate_limit_config: RateLimitConfig) -> &'static Self {
    static LOCAL_INSTANCE: OnceCell<RateLimitCell> = OnceCell::const_new();
    LOCAL_INSTANCE
      .get_or_init(|| async {
        let (tx, mut rx) = mpsc::channel::<RateLimitConfig>(4);
        let rate_limit = Arc::new(Mutex::new(RateLimit {
          rate_limiter: RateLimitStorage::new(rate_limit_config.into()),
        }));
        let rate_limit2 = rate_limit.clone();
        tokio::spawn(async move {
          while let Some(r) = rx.recv().await {
            rate_limit2
              .lock()
              .expect("Failed to lock rate limit mutex for updating")
              .rate_limiter
              .set_config(r.into());
          }
        });
        RateLimitCell { tx, rate_limit }
      })
      .await
  }

  /// Call this when the config was updated, to update all in-memory cells.
  pub async fn send(&self, config: RateLimitConfig) -> Result<(), LemmyError> {
    self.tx.send(config).await?;
    Ok(())
  }

  pub fn remove_full_buckets(&self) {
    let mut guard = self
      .rate_limit
      .lock()
      .expect("Failed to lock rate limit mutex for reading");

    guard.rate_limiter.remove_full_buckets(InstantSecs::now())
  }

  pub fn message(&self) -> RateLimitedGuard {
    self.kind(RateLimitType::Message)
  }

  pub fn post(&self) -> RateLimitedGuard {
    self.kind(RateLimitType::Post)
  }

  pub fn register(&self) -> RateLimitedGuard {
    self.kind(RateLimitType::Register)
  }

  pub fn image(&self) -> RateLimitedGuard {
    self.kind(RateLimitType::Image)
  }

  pub fn comment(&self) -> RateLimitedGuard {
    self.kind(RateLimitType::Comment)
  }

  pub fn search(&self) -> RateLimitedGuard {
    self.kind(RateLimitType::Search)
  }

  fn kind(&self, type_: RateLimitType) -> RateLimitedGuard {
    RateLimitedGuard {
      rate_limit: self.rate_limit.clone(),
      type_,
    }
  }
}

pub struct RateLimitedMiddleware<S> {
  rate_limited: RateLimitedGuard,
  service: Rc<S>,
}

impl RateLimitedGuard {
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub fn check(self, ip_addr: IpAddr) -> bool {
    // Does not need to be blocking because the RwLock in settings never held across await points,
    // and the operation here locks only long enough to clone
    let mut guard = self
      .rate_limit
      .lock()
      .expect("Failed to lock rate limit mutex for reading");

    let limiter = &mut guard.rate_limiter;

    limiter.check_rate_limit_full(self.type_, ip_addr, InstantSecs::now())
  }
}

impl<S> Transform<S, ServiceRequest> for RateLimitedGuard
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
        Ok(ServiceResponse::from_err(
          LemmyError::from(LemmyErrorType::RateLimitError),
          http_req,
        ))
      }
    })
  }
}

fn get_ip(conn_info: &ConnectionInfo) -> IpAddr {
  conn_info
    .realip_remote_addr()
    .and_then(parse_ip)
    .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
}

fn parse_ip(addr: &str) -> Option<IpAddr> {
  if let Some(s) = addr.strip_suffix(']') {
    IpAddr::from_str(s.get(1..)?).ok()
  } else if let Ok(ip) = IpAddr::from_str(addr) {
    Some(ip)
  } else if let Ok(socket) = SocketAddr::from_str(addr) {
    Some(socket.ip())
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  #[test]
  fn test_parse_ip() {
    let ip_addrs = [
      "1.2.3.4",
      "1.2.3.4:8000",
      "2001:db8::",
      "[2001:db8::]",
      "[2001:db8::]:8000",
    ];
    for addr in ip_addrs {
      assert!(super::parse_ip(addr).is_some(), "failed to parse {addr}");
    }
  }
}
