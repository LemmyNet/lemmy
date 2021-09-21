use crate::blocking;
use chrono::Utc;
use diesel::PgConnection;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use lazy_static::lazy_static;
use lemmy_db_queries::{source::secrets::Secrets_, DbPool};
use lemmy_db_schema::source::secrets::Secrets;
use lemmy_utils::{settings::structs::Settings, LemmyError};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::RwLock};

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
    let secret = blocking(pool, move |conn| get_jwt_secret(conn)).await??;
    let key = DecodingKey::from_secret(secret.as_ref());
    Ok(decode::<Claims>(jwt, &key, &v)?)
  }

  pub async fn jwt(local_user_id: i32, pool: &DbPool) -> Result<Jwt, LemmyError> {
    let my_claims = Claims {
      sub: local_user_id,
      iss: Settings::get().hostname,
      iat: Utc::now().timestamp(),
    };

    let secret = blocking(pool, move |conn| get_jwt_secret(conn)).await??;
    let key = EncodingKey::from_secret(secret.as_ref());
    Ok(encode(&Header::default(), &my_claims, &key)?)
  }
}

lazy_static! {
  static ref JWT_SECRET: RwLock<Option<String>> = RwLock::new(None);
}

fn get_jwt_secret(conn: &PgConnection) -> Result<String, LemmyError> {
  let jwt_option: Option<String> = JWT_SECRET.read().unwrap().deref().clone();
  match jwt_option {
    Some(j) => Ok(j),
    None => {
      let jwt = Secrets::read(conn)?;
      let jwt_static = JWT_SECRET.write();
      let mut jwt_static = jwt_static.unwrap();
      *jwt_static = Some(jwt.clone());
      Ok(jwt)
    }
  }
}
