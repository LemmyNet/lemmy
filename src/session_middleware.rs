use actix_web::{
  body::MessageBody,
  cookie::SameSite,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::header::{Header, CACHE_CONTROL},
  Error,
  HttpMessage,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::{
  context::LemmyContext,
  lemmy_db_views::structs::MaybeLocalUserView,
  utils::local_user_view_from_jwt,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
use reqwest::header::HeaderValue;
use std::{future::ready, rc::Rc};

static AUTH_COOKIE_NAME: &str = "auth";

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
      // Try reading jwt from auth header
      let auth_header = Authorization::<Bearer>::parse(&req).ok();
      let jwt = if let Some(a) = auth_header {
        println!("Got token: {}", a.clone().as_ref().token());
        Some(a.as_ref().token().to_string())
      }
      // If that fails, try auth cookie. Dont use the `jwt` cookie from lemmy-ui because
      // its not http-only.
      else {
        let auth_cookie = req.cookie(AUTH_COOKIE_NAME);
        if let Some(a) = &auth_cookie {
          // ensure that its marked as httponly and secure
          let secure = a.secure().unwrap_or_default();
          let http_only = a.http_only().unwrap_or_default();
          let same_site = a.same_site();
          if !secure || !http_only || same_site != Some(SameSite::Strict) {
            return Err(LemmyError::from(LemmyErrorType::AuthCookieInsecure).into());
          }
        }
        auth_cookie.map(|c| c.value().to_string())
      };

      let maybe_local_user_view = if let Some(jwt) = &jwt {
        // Ignore any invalid auth so the site can still be used
        // TODO: this means it will be impossible to get any error message for invalid jwt. Need
        //       to add a separate endpoint for that.
        //       https://github.com/LemmyNet/lemmy/issues/3702
        local_user_view_from_jwt(jwt, &context).await.ok()
      } else {
        None
      };

      req
        .extensions_mut()
        .insert(MaybeLocalUserView(maybe_local_user_view));

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
