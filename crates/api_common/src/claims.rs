use crate::{context::LemmyContext, sensitive::Sensitive};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::login_token::{LoginToken, LoginTokenCreateForm},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
      return Err(LemmyErrorType::NotLoggedIn)?;
    }
    Ok(user_id)
  }

  pub async fn generate(
    user_id: LocalUserId,
    context: &LemmyContext,
  ) -> LemmyResult<Sensitive<String>> {
    let hostname = context.settings().hostname.clone();
    let my_claims = Claims {
      sub: user_id.0.to_string(),
      iss: hostname,
      iat: Utc::now().timestamp(),
    };

    let secret = &context.secret().jwt_secret;
    let key = EncodingKey::from_secret(secret.as_ref());
    let token = encode(&Header::default(), &my_claims, &key)?;
    let form = LoginTokenCreateForm {
      token: token.clone(),
      user_id,
    };
    LoginToken::create(&mut context.pool(), form).await?;
    Ok(Sensitive::new(token))
  }
}
