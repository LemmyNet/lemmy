use actix_web::{
  body::MessageBody,
  cookie::SameSite,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::header::CACHE_CONTROL,
  Error,
  HttpMessage,
};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::{context::LemmyContext, utils::local_user_view_from_jwt};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
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
      let mut is_auth = false;
      // Try reading jwt from auth header
      let auth_header = req.headers().get("auth").map(|h| h.to_str().ok()).flatten();
      if let Some(a) = auth_header {
        let local_user_view = local_user_view_from_jwt(a, &context).await?;
        req.extensions_mut().insert(local_user_view);
        is_auth = true;
      }
      // If that fails, try auth cookie. Dont use the `jwt` cookie from lemmy-ui because
      // its not http-only.
      else {
        let auth_cookie = req.cookie("auth");
        if let Some(a) = auth_cookie {
          // ensure that its marked as httponly and secure
          let secure = a.secure().unwrap_or_default();
          let http_only = a.http_only().unwrap_or_default();
          let same_site = a.same_site();
          if !secure || !http_only || same_site != Some(SameSite::Strict) {
            return Err(LemmyError::from(LemmyErrorType::AuthCookieInsecure).into());
          }
          let local_user_view = local_user_view_from_jwt(a.value(), &context).await?;
          req.extensions_mut().insert(local_user_view);
          is_auth = true;
        }
      }

      let mut res = svc.call(req).await?;

      // Add cache-control header. If user is authenticated, mark as private. Otherwise cache
      // up to one minute.
      let cache_value = if is_auth {
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
