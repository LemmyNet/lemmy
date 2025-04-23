use actix_web::{
  body::EitherBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::Method,
  Error,
  HttpMessage,
  HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::rate_limit::rate_limiter::InstantSecs;
use std::{
  collections::HashSet,
  future::{ready, Ready},
  hash::{Hash, Hasher},
  sync::{Arc, RwLock},
  time::Duration,
};

/// https://www.ietf.org/archive/id/draft-ietf-httpapi-idempotency-key-header-01.html
const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";

/// Delete idempotency keys older than this
const CLEANUP_INTERVAL_SECS: u32 = 120;

#[derive(Debug)]
struct Entry {
  user_id: LocalUserId,
  key: String,
  // Creation time is ignored for Eq, Hash and only used to cleanup old entries
  created: InstantSecs,
}

impl PartialEq for Entry {
  fn eq(&self, other: &Self) -> bool {
    self.user_id == other.user_id && self.key == other.key
  }
}
impl Eq for Entry {}

impl Hash for Entry {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.user_id.hash(state);
    self.key.hash(state);
  }
}

#[derive(Clone)]
pub struct IdempotencySet {
  set: Arc<RwLock<HashSet<Entry>>>,
}

impl Default for IdempotencySet {
  fn default() -> Self {
    let set: Arc<RwLock<HashSet<Entry>>> = Default::default();

    let set_ = set.clone();
    tokio::spawn(async move {
      let interval = Duration::from_secs(CLEANUP_INTERVAL_SECS.into());
      let state_weak_ref = Arc::downgrade(&set_);

      // Run at every interval to delete entries older than the interval.
      // This loop stops when all other references to `state` are dropped.
      while let Some(state) = state_weak_ref.upgrade() {
        tokio::time::sleep(interval).await;
        let now = InstantSecs::now();
        #[allow(clippy::expect_used)]
        let mut lock = state.write().expect("lock failed");
        lock.retain(|e| e.created.secs > now.secs.saturating_sub(CLEANUP_INTERVAL_SECS));
        lock.shrink_to_fit();
      }
    });
    Self { set }
  }
}

pub struct IdempotencyMiddleware {
  idempotency_set: IdempotencySet,
}

impl IdempotencyMiddleware {
  pub fn new(idempotency_set: IdempotencySet) -> Self {
    Self { idempotency_set }
  }
}

impl<S, B> Transform<S, ServiceRequest> for IdempotencyMiddleware
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type InitError = ();
  type Transform = IdempotencyService<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(IdempotencyService {
      service,
      idempotency_set: self.idempotency_set.clone(),
    }))
  }
}

pub struct IdempotencyService<S> {
  service: S,
  idempotency_set: IdempotencySet,
}

impl<S, B> Service<ServiceRequest> for IdempotencyService<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<EitherBody<B>>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  #[allow(clippy::expect_used)]
  fn call(&self, req: ServiceRequest) -> Self::Future {
    let is_post_or_put = req.method() == Method::POST || req.method() == Method::PUT;
    let idempotency = req
      .headers()
      .get(IDEMPOTENCY_HEADER)
      .map(|i| i.to_str().unwrap_or_default().to_string())
      // Ignore values longer than 32 chars
      .and_then(|i| (i.len() <= 32).then_some(i))
      // Only use idempotency for POST and PUT requests
      .and_then(|i| is_post_or_put.then_some(i));

    let user_id = {
      let ext = req.extensions();
      ext.get().map(|u: &LocalUserView| u.local_user.id)
    };

    if let (Some(key), Some(user_id)) = (idempotency, user_id) {
      let value = Entry {
        user_id,
        key,
        created: InstantSecs::now(),
      };
      if self
        .idempotency_set
        .set
        .read()
        .expect("lock failed")
        .contains(&value)
      {
        // Duplicate request, return error
        let (req, _pl) = req.into_parts();
        let response = HttpResponse::UnprocessableEntity()
          .finish()
          .map_into_right_body();
        return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
      } else {
        // New request, store key and continue
        self
          .idempotency_set
          .set
          .write()
          .expect("lock failed")
          .insert(value);
      }
    }

    let fut = self.service.call(req);

    Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) })
  }
}
