use actix_web::{
  body::MessageBody,
  cookie::SameSite,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::header::CACHE_CONTROL,
  Error,
  HttpMessage,
};
use chrono::{DateTime, Utc};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::{
  context::LemmyContext,
  lemmy_db_views::structs::LocalUserView,
  utils::check_user_valid,
};
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_utils::{
  claims::Claims,
  error::{LemmyError, LemmyErrorExt2, LemmyErrorType},
};
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
      let auth_header = req
        .headers()
        .get(AUTH_COOKIE_NAME)
        .and_then(|h| h.to_str().ok());
      let jwt = if let Some(a) = auth_header {
        Some(a.to_string())
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
  let claims = Claims::decode(jwt, &context.secret().jwt_secret)
    .with_lemmy_type(LemmyErrorType::NotLoggedIn)?
    .claims;
  let local_user_id = LocalUserId(claims.sub);
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;
  check_user_valid(
    local_user_view.person.banned,
    local_user_view.person.ban_expires,
    local_user_view.person.deleted,
  )?;

  check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

  Ok(local_user_view)
}

/// Checks if user's token was issued before user's password reset.
fn check_validator_time(validator_time: &DateTime<Utc>, claims: &Claims) -> Result<(), LemmyError> {
  let user_validation_time = validator_time.timestamp();
  if user_validation_time > claims.iat {
    Err(LemmyErrorType::NotLoggedIn)?
  } else {
    Ok(())
  }
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
  use lemmy_utils::{claims::Claims, settings::SETTINGS};
  use serial_test::serial;
  use std::env;

  #[tokio::test]
  #[serial]
  async fn test_should_not_validate_user_token_after_password_change() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let secret = Secret::init(pool).await.unwrap();

    // test.sh sets `LEMMY_CONFIG_LOCATION=../../config/config.hjson` for code under crates folder.
    // this results in a config not found error, so we need to unset this var and use default.
    env::remove_var("LEMMY_CONFIG_LOCATION");
    let settings = &SETTINGS.to_owned();

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

    let jwt = Claims::jwt(
      inserted_local_user.id.0,
      &secret.jwt_secret,
      &settings.hostname,
    )
    .unwrap();
    let claims = Claims::decode(&jwt, &secret.jwt_secret).unwrap().claims;
    let check = check_validator_time(&inserted_local_user.validator_time, &claims);
    assert!(check.is_ok());

    // The check should fail, since the validator time is now newer than the jwt issue time
    let updated_local_user =
      LocalUser::update_password(pool, inserted_local_user.id, "password111")
        .await
        .unwrap();
    let check_after = check_validator_time(&updated_local_user.validator_time, &claims);
    assert!(check_after.is_err());

    let num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    assert_eq!(1, num_deleted);
  }
}
