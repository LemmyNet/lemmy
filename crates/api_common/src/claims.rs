use crate::blocking;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use lemmy_db_queries::{source::secrets::Secrets_, DbPool};
use lemmy_db_schema::source::secrets::Secrets;
use lemmy_utils::{settings::structs::Settings, LemmyError};
use serde::{Deserialize, Serialize};

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  /// local_user_id, standard claim by RFC 7519.
  pub sub: i32,
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
}

impl Claims {
  pub async fn decode(jwt: &str, pool: &DbPool) -> Result<TokenData<Claims>, LemmyError> {
    let v = Validation {
      validate_exp: false,
      ..Validation::default()
    };
    let secret = get_jwt_secret(pool).await?;
    let key = DecodingKey::from_secret(secret.as_ref());
    Ok(decode::<Claims>(jwt, &key, &v)?)
  }

  pub async fn jwt(local_user_id: i32, pool: &DbPool) -> Result<Jwt, LemmyError> {
    let my_claims = Claims {
      sub: local_user_id,
      iss: Settings::get().hostname,
      iat: Utc::now().timestamp(),
    };
    let key = EncodingKey::from_secret(get_jwt_secret(pool).await?.as_ref());
    Ok(encode(&Header::default(), &my_claims, &key)?)
  }
}

/// TODO: would be good if we could store the jwt secret in memory, so we dont have to run db
///       queries all the time (which probably affects performance). but its tricky, we cant use a
///       static because it requires a db connection to initialize.
async fn get_jwt_secret(pool: &DbPool) -> Result<String, LemmyError> {
  let jwt_secret = blocking(pool, move |conn| Secrets::read(conn)).await??;
  Ok(jwt_secret)
}
