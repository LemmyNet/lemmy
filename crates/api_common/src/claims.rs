use crate::context::LemmyContext;
use actix_web::{http::header::USER_AGENT, HttpRequest};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
}

impl Claims {
  pub async fn validate(jwt: &str, context: &LemmyContext) -> LemmyResult<LocalUserId> {
    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation.required_spec_claims.remove("exp");
    let jwt_secret = &context.secret().jwt_secret;
    let key = DecodingKey::from_secret(jwt_secret.as_ref());
    let claims =
      decode::<Claims>(jwt, &key, &validation).with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
    let user_id = LocalUserId(claims.claims.sub.parse()?);
    let is_valid = LoginToken::validate(&mut context.pool(), user_id, jwt).await?;
    if !is_valid {
      Err(LemmyErrorType::NotLoggedIn)?
    } else {
      Ok(user_id)
    }
  }

  pub async fn generate(
    user_id: LocalUserId,
    req: HttpRequest,
    context: &LemmyContext,
  ) -> LemmyResult<SensitiveString> {
    let hostname = context.settings().hostname.clone();
    let my_claims = Claims {
      sub: user_id.0.to_string(),
      iss: hostname,
      iat: Utc::now().timestamp(),
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{claims::Claims, context::LemmyContext};
  use actix_web::test::TestRequest;
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
  use lemmy_utils::rate_limit::RateLimitCell;
  use pretty_assertions::assert_eq;
  use reqwest::Client;
  use reqwest_middleware::ClientBuilder;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_should_not_validate_user_token_after_password_change() {
    let pool_ = build_db_pool_for_tests().await;
    let pool = &mut (&pool_).into();
    let secret = Secret::init(pool).await.unwrap().unwrap();
    let context = LemmyContext::create(
      pool_.clone(),
      ClientBuilder::new(Client::default()).build(),
      secret,
      RateLimitCell::with_test_config(),
    );

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "Gerry9812");

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let local_user_form = LocalUserInsertForm::test_form(inserted_person.id);

    let inserted_local_user = LocalUser::create(pool, &local_user_form, vec![])
      .await
      .unwrap();

    let req = TestRequest::default().to_http_request();
    let jwt = Claims::generate(inserted_local_user.id, req, &context)
      .await
      .unwrap();

    let valid = Claims::validate(&jwt, &context).await;
    assert!(valid.is_ok());

    let num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    assert_eq!(1, num_deleted);
  }
}
