use actix_web::{
  body::MessageBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::header::CACHE_CONTROL,
  Error,
  HttpMessage,
};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api::read_auth_token;
use lemmy_api_common::{context::LemmyContext, utils::local_user_view_from_jwt};
use reqwest::header::HeaderValue;
use std::{future::ready, rc::Rc};

#[derive(Clone)]
pub struct SessionMiddleware {
  context: LemmyContext,
}

impl SessionMiddleware {
  pub fn new(context: LemmyContext) -> Self {
    SessionMiddleware { context }
  }
}
impl<S, B> Transform<S, ServiceRequest> for SessionMiddleware
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: MessageBody + 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Transform = SessionService<S>;
  type InitError = ();
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(SessionService {
      service: Rc::new(service),
      context: self.context.clone(),
    }))
  }
}

pub struct SessionService<S> {
  service: Rc<S>,
  context: LemmyContext,
}

impl<S, B> Service<ServiceRequest> for SessionService<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let svc = self.service.clone();
    let context = self.context.clone();

    Box::pin(async move {
      let jwt = read_auth_token(req.request())?;

      if let Some(jwt) = &jwt {
        // Ignore any invalid auth so the site can still be used
        // TODO: this means it will be impossible to get any error message for invalid jwt. Need
        //       to add a separate endpoint for that.
        //       https://github.com/LemmyNet/lemmy/issues/3702
        let local_user_view = local_user_view_from_jwt(jwt, &context).await.ok();
        if let Some(local_user_view) = local_user_view {
          req.extensions_mut().insert(local_user_view);
        }
      }

      let mut res = svc.call(req).await?;

      // Add cache-control header. If user is authenticated, mark as private. Otherwise cache
      // up to one minute.
      let cache_value = if jwt.is_some() {
        "private"
      } else {
        "public, max-age=60"
      };
      res
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(cache_value));
      Ok(res)
    })
  }
}
