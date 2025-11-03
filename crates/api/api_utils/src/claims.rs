use crate::context::LemmyContext;
use actix_web::{HttpRequest, http::header::USER_AGENT};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  sensitive::SensitiveString,
  source::login_token::{LoginToken, LoginTokenCreateForm},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Claims {
  /// local_user_id, standard claim by RFC 7519.
  pub sub: String,
  /// Server domain
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
  /// Expiration timestamp
  pub exp: i64,
}

impl Claims {
  pub async fn validate(jwt: &str, context: &LemmyContext) -> LemmyResult<LocalUserId> {
    let validation = Validation::default();
    let jwt_secret = &context.secret().jwt_secret;
    let key = DecodingKey::from_secret(jwt_secret.as_ref());
    let claims =
      decode::<Claims>(jwt, &key, &validation).with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
    let user_id = LocalUserId(claims.claims.sub.parse()?);
    LoginToken::validate(&mut context.pool(), user_id, jwt).await?;
    Ok(user_id)
  }

  pub async fn generate(
    user_id: LocalUserId,
    stay_logged_in: Option<bool>,
    req: HttpRequest,
    context: &LemmyContext,
  ) -> LemmyResult<SensitiveString> {
    let hostname = context.settings().hostname.clone();
    let now = Utc::now();
    let exp = if stay_logged_in.unwrap_or_default() {
      // Login doesnt expire
      DateTime::<Utc>::MAX_UTC
    } else {
      // Login expires after one week
      now + Duration::weeks(1)
    };
    let my_claims = Claims {
      sub: user_id.0.to_string(),
      iss: hostname,
      iat: now.timestamp(),
      exp: exp.timestamp(),
    };

    let secret = &context.secret().jwt_secret;
    let key = EncodingKey::from_secret(secret.as_ref());
    let token: SensitiveString = encode(&Header::default(), &my_claims, &key)?.into();
    let ip = req
      .connection_info()
      .realip_remote_addr()
      .map(ToString::to_string);
    let user_agent = req
      .headers()
      .get(USER_AGENT)
      .and_then(|ua| ua.to_str().ok())
      .map(ToString::to_string);
    let form = LoginTokenCreateForm {
      token: token.clone(),
      user_id,
      ip,
      user_agent,
    };
    LoginToken::create(&mut context.pool(), form).await?;
    Ok(token)
  }
}

#[cfg(test)]
mod tests {

  use crate::{claims::Claims, context::LemmyContext};
  use actix_web::test::TestRequest;
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
  async fn test_should_not_validate_user_token_after_password_change() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "Gerry9812");

    let inserted_person = Person::create(pool, &new_person).await?;

    let local_user_form = LocalUserInsertForm::test_form(inserted_person.id);

    let inserted_local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;

    let req = TestRequest::default().to_http_request();
    let jwt = Claims::generate(inserted_local_user.id, None, req, &context).await?;

    let valid = Claims::validate(&jwt, &context).await;
    assert!(valid.is_ok());

    let num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
