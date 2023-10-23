use crate::error::{LemmyError, LemmyErrorType};
use actix_web::dev::{ConnectionInfo, Service, ServiceRequest, ServiceResponse, Transform};
use enum_map::{enum_map, EnumMap};
use futures::future::{ok, Ready};
pub use rate_limiter::{ActionType, BucketConfig};
use rate_limiter::{InstantSecs, RateLimitState};
use std::{
  future::Future,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  pin::Pin,
  rc::Rc,
  str::FromStr,
  sync::{Arc, Mutex},
  task::{Context, Poll},
  time::Duration,
};

pub mod rate_limiter;

#[derive(Debug, Clone)]
pub struct RateLimitChecker {
  state: Arc<Mutex<RateLimitState>>,
  action_type: ActionType,
}

/// Single instance of rate limit config and buckets, which is shared across all threads.
#[derive(Clone)]
pub struct RateLimitCell {
  state: Arc<Mutex<RateLimitState>>,
}

impl RateLimitCell {
  pub fn new(rate_limit_config: EnumMap<ActionType, BucketConfig>) -> Self {
    let state = Arc::new(Mutex::new(RateLimitState::new(rate_limit_config)));

    let state_weak_ref = Arc::downgrade(&state);

    tokio::spawn(async move {
      let hour = Duration::from_secs(3600);

      // This loop stops when all other references to `state` are dropped
      while let Some(state) = state_weak_ref.upgrade() {
        tokio::time::sleep(hour).await;
        state
          .lock()
          .expect("Failed to lock rate limit mutex for reading")
          .remove_full_buckets(InstantSecs::now());
      }
    });

    RateLimitCell { state }
  }

  pub fn set_config(&self, config: EnumMap<ActionType, BucketConfig>) {
    self
      .state
      .lock()
      .expect("Failed to lock rate limit mutex for updating")
      .set_config(config);
  }

  pub fn message(&self) -> RateLimitChecker {
    self.new_checker(ActionType::Message)
  }

  pub fn post(&self) -> RateLimitChecker {
    self.new_checker(ActionType::Post)
  }

  pub fn register(&self) -> RateLimitChecker {
    self.new_checker(ActionType::Register)
  }

  pub fn image(&self) -> RateLimitChecker {
    self.new_checker(ActionType::Image)
  }

  pub fn comment(&self) -> RateLimitChecker {
    self.new_checker(ActionType::Comment)
  }

  pub fn search(&self) -> RateLimitChecker {
    self.new_checker(ActionType::Search)
  }

  pub fn import_user_settings(&self) -> RateLimitChecker {
    self.new_checker(ActionType::ImportUserSettings)
  }

  fn new_checker(&self, action_type: ActionType) -> RateLimitChecker {
    RateLimitChecker {
      state: self.state.clone(),
      action_type,
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
}

pub struct RateLimitedMiddleware<S> {
  checker: RateLimitChecker,
  service: Rc<S>,
}

impl RateLimitChecker {
  /// Returns true if the request passed the rate limit, false if it failed and should be rejected.
  pub fn check(self, ip_addr: IpAddr) -> bool {
    // Does not need to be blocking because the RwLock in settings never held across await points,
    // and the operation here locks only long enough to clone
    let mut state = self
      .state
      .lock()
      .expect("Failed to lock rate limit mutex for reading");

    state.check(self.action_type, ip_addr, InstantSecs::now())
  }
}

impl<S> Transform<S, ServiceRequest> for RateLimitChecker
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
      checker: self.clone(),
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

    let checker = self.checker.clone();
    let service = self.service.clone();

    Box::pin(async move {
      if checker.check(ip_addr) {
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
