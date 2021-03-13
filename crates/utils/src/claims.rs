use crate::settings::Settings;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  /// User id, for backward compatibility with client apps.
  /// Claim [sub](Claims::sub) is used in server-side checks.
  pub id: i32,
  /// User id, standard claim by RFC 7519.
  pub sub: i32,
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
}

impl Claims {
  pub fn decode(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let v = Validation {
      validate_exp: false,
      ..Validation::default()
    };
    decode::<Claims>(
      &jwt,
      &DecodingKey::from_secret(Settings::get().jwt_secret.as_ref()),
      &v,
    )
  }

  pub fn jwt(user_id: i32, hostname: String) -> Result<Jwt, jsonwebtoken::errors::Error> {
    let my_claims = Claims {
      id: user_id,
      sub: user_id,
      iss: hostname,
      iat: chrono::Utc::now().timestamp_millis() / 1000,
    };
    encode(
      &Header::default(),
      &my_claims,
      &EncodingKey::from_secret(Settings::get().jwt_secret.as_ref()),
    )
  }
}
