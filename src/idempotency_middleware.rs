use actix_web::{
  body::EitherBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::Method,
  Error,
  HttpMessage,
  HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::lemmy_db_views::structs::LocalUserView;
use lemmy_db_schema::newtypes::LocalUserId;
use std::{
  collections::HashSet,
  future::{ready, Ready},
  sync::{Arc, RwLock},
};

/// https://www.ietf.org/archive/id/draft-ietf-httpapi-idempotency-key-header-01.html
const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";

// TODO: cleanup entries after a while
pub type IdempotencySet = Arc<RwLock<HashSet<(LocalUserId, String)>>>;

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

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let is_post_or_put = req.method() == Method::POST || req.method() == Method::PUT;
    let idempotency = req
      .headers()
      .get(IDEMPOTENCY_HEADER)
      .map(|i| i.to_str().unwrap_or_default().to_string())
      // Ignore values longer than 32 chars
      .map(|i| (i.len() <= 32).then_some(i))
      .flatten()
      // Only use idempotency for POST and PUT requests
      .map(|i| is_post_or_put.then_some(i))
      .flatten();

    let user_id = {
      let ext = req.extensions();
      ext.get().map(|u: &LocalUserView| u.local_user.id)
    };

    if let (Some(idempotency), Some(user_id)) = (idempotency, user_id) {
      let value = (user_id, idempotency);
      if self.idempotency_set.read().unwrap().contains(&value) {
        // Duplicate request, return error
        let (req, _pl) = req.into_parts();
        // TODO: need to return LemmyError as well?
        let response = HttpResponse::UnprocessableEntity()
          .finish()
          .map_into_right_body();
        return Box::pin(async { Ok(ServiceResponse::new(req, response)) });
      } else {
        // New request, store key and continue
        self.idempotency_set.write().unwrap().insert(value);
      }
    }

    let fut = self.service.call(req);

    Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) })
  }
}
