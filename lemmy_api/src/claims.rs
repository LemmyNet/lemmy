use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use lemmy_db::user::User_;
use lemmy_utils::settings::Settings;
use serde::{Deserialize, Serialize};

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
/// The API claims
pub struct Claims {
  /// The user id
  pub id: i32,
  /// The issuer
  pub iss: String,
}

impl Claims {
  /// Decode a jwt into claims
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

  /// Encode a jwt from a user
  pub fn jwt(user: User_, hostname: String) -> Result<Jwt, jsonwebtoken::errors::Error> {
    let my_claims = Claims {
      id: user.id,
      iss: hostname,
    };
    encode(
      &Header::default(),
      &my_claims,
      &EncodingKey::from_secret(Settings::get().jwt_secret.as_ref()),
    )
  }
}
