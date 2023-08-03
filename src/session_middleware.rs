use actix_web::{
  body::MessageBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  HttpMessage,
};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::{context::LemmyContext, utils::local_user_view_from_jwt};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
use std::{future::ready, rc::Rc};

#[derive(Clone)]
pub struct SessionMiddleware {
  context: LemmyContext,
}

impl<S, B> Transform<S, ServiceRequest> for SessionMiddleware
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = LemmyError> + 'static,
  S::Future: 'static,
  B: MessageBody + 'static,
{
  type Response = ServiceResponse<B>;
  type Error = LemmyError;
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
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = LemmyError> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = LemmyError;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let svc = self.service.clone();
    let context = self.context.clone();

    Box::pin(async move {
      // Try reading jwt from auth header
      let auth_header = req.headers().get("auth").map(|h| h.to_str().ok()).flatten();
      if let Some(a) = auth_header {
        let local_user_view = local_user_view_from_jwt(a, &context).await?;
        req.extensions_mut().insert(local_user_view);
      }
      // If that fails, try auth cookie. Dont use the `jwt` cookie from lemmy-ui because
      // its not http-only.
      else {
        let auth_cookie = req.cookie("auth");
        if let Some(a) = auth_cookie {
          // ensure that its marked as httponly and secure
          let secure = a.secure().unwrap_or_default();
          let http_only = a.http_only().unwrap_or_default();
          if !secure || !http_only {
            return Err(LemmyErrorType::JwtCookieInsecure.into());
          }
          let local_user_view = local_user_view_from_jwt(a.value(), &context).await?;
          req.extensions_mut().insert(local_user_view);
        }
      }

      svc.call(req).await
    })
  }
}
