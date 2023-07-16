use crate::error::LemmyError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthMethod {
  Password,
  Api,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  /// local_user_id, standard claim by RFC 7519.
  pub sub: i32,
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
  // TODO: This should be made non-optional once deprecated /login endpoint has been removed
  pub exp: Option<i64>,
  // TODO: This should be made non-optional once deprecated /login endpoint has been removed
  pub method: Option<AuthMethod>,
}

impl Claims {
  pub fn decode(jwt: &str, jwt_secret: &str) -> Result<TokenData<Claims>, LemmyError> {
    let mut validation = Validation::default();
    let key = DecodingKey::from_secret(jwt_secret.as_ref());

    let decoded_token = match decode::<Claims>(jwt, &key, &validation) {
      Ok(res) => res,
      Err(_) => {
        // For backwards compatibility with deprecated authentication, we also allow JWTs with no
        // expiry.
        // TODO: This should be removed once the deprecated /login endpoint has been removed
        validation.validate_exp = false;
        validation.required_spec_claims.remove("exp");
        decode::<Claims>(jwt, &key, &validation)?
      }
    };

    Ok(decoded_token)
  }

  // Used for deprecated /login endpoint, does not have an expiry
  // TODO: This should be removed once the deprecated /login endpoint has been removed
  pub fn jwt(local_user_id: i32, jwt_secret: &str, hostname: &str) -> Result<Jwt, LemmyError> {
    let my_claims = Claims {
      sub: local_user_id,
      iss: hostname.to_string(),
      iat: Utc::now().timestamp(),
      exp: None,
      method: None,
    };

    let key = EncodingKey::from_secret(jwt_secret.as_ref());
    Ok(encode(&Header::default(), &my_claims, &key)?)
  }

  pub fn jwt_with_exp(
    local_user_id: i32,
    jwt_secret: &str,
    hostname: &str,
    method: AuthMethod,
  ) -> Result<Jwt, LemmyError> {
    let my_claims = Claims {
      sub: local_user_id,
      iss: hostname.to_string(),
      iat: Utc::now().timestamp(),
      exp: Some((Utc::now() + Duration::minutes(5)).timestamp()),
      method: Some(method),
    };

    let key = EncodingKey::from_secret(jwt_secret.as_ref());
    Ok(encode(&Header::default(), &my_claims, &key)?)
  }
}
