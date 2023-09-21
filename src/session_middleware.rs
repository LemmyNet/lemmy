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
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  lemmy_db_views::structs::LocalUserView,
  utils::check_user_valid,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt2, LemmyErrorType};
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

#[tracing::instrument(skip_all)]
async fn local_user_view_from_jwt(
  jwt: &str,
  context: &LemmyContext,
) -> Result<LocalUserView, LemmyError> {
  let local_user_id = Claims::validate(jwt, context)
    .await
    .with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;
  check_user_valid(
    local_user_view.person.banned,
    local_user_view.person.ban_expires,
    local_user_view.person.deleted,
  )?;

  Ok(local_user_view)
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      secret::Secret,
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::rate_limit::{RateLimitCell, RateLimitConfig};
  use reqwest::Client;
  use reqwest_middleware::ClientBuilder;
  use serial_test::serial;
  use std::env::set_current_dir;

  #[tokio::test]
  #[serial]
  async fn test_session_auth() {
    // hack, necessary so that config file can be loaded from hardcoded, relative path
    set_current_dir("crates/utils").unwrap();

    let pool_ = build_db_pool_for_tests().await;
    let pool = &mut (&pool_).into();
    let secret = Secret::init(pool).await.unwrap();
    let context = LemmyContext::create(
      pool_.clone(),
      ClientBuilder::new(Client::default()).build(),
      secret,
      RateLimitCell::new(RateLimitConfig::builder().build())
        .await
        .clone(),
    );

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("Gerry9812".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted("123456".to_string())
      .build();

    let inserted_local_user = LocalUser::create(pool, &local_user_form).await.unwrap();

    let jwt = Claims::generate(inserted_local_user.id, &context)
      .await
      .unwrap();

    let valid = Claims::validate(&jwt, &context).await;
    assert!(valid.is_ok());

    let num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    assert_eq!(1, num_deleted);
  }
}
