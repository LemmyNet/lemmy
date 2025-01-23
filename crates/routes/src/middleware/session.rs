use actix_web::{
  body::MessageBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::header::{HeaderValue, CACHE_CONTROL},
  Error,
  HttpMessage,
};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::{
  context::LemmyContext,
  utils::{local_user_view_from_jwt, read_auth_token},
};
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
        // This means it is be impossible to get any error message for invalid jwt. Need
        // to use `/api/v4/account/validate_auth` for that.
        let local_user_view = local_user_view_from_jwt(jwt, &context).await.ok();
        if let Some(local_user_view) = local_user_view {
          req.extensions_mut().insert(local_user_view);
        }
      }

      let mut res = svc.call(req).await?;

      // Add cache-control header if none is present
      if !res.headers().contains_key(CACHE_CONTROL) {
        // If user is authenticated, mark as private. Otherwise cache
        // up to one minute.
        let cache_value = if jwt.is_some() {
          "private"
        } else {
          "public, max-age=60"
        };
        res
          .headers_mut()
          .insert(CACHE_CONTROL, HeaderValue::from_static(cache_value));
      }
      Ok(res)
    })
  }
}

#[cfg(test)]
mod tests {

  use actix_web::test::TestRequest;
  use lemmy_api_common::{claims::Claims, context::LemmyContext};
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_session_auth() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;

    let inserted_instance =
      Instance::read_or_create(&mut context.pool(), "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "Gerry9812");

    let inserted_person = Person::create(&mut context.pool(), &new_person).await?;

    let local_user_form = LocalUserInsertForm::test_form(inserted_person.id);

    let inserted_local_user =
      LocalUser::create(&mut context.pool(), &local_user_form, vec![]).await?;

    let req = TestRequest::default().to_http_request();
    let jwt = Claims::generate(inserted_local_user.id, req, &context).await?;

    let valid = Claims::validate(&jwt, &context).await;
    assert!(valid.is_ok());

    let num_deleted = Person::delete(&mut context.pool(), inserted_person.id).await?;
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
