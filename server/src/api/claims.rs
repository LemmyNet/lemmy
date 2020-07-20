use diesel::{result::Error, PgConnection};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use lemmy_db::{user::User_, Crud};
use lemmy_utils::settings::Settings;
use serde::{Deserialize, Serialize};

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub id: i32,
  pub username: String,
  pub iss: String,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub avatar: Option<String>,
  pub show_avatars: bool,
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

  pub fn jwt(user: User_, hostname: String) -> Jwt {
    let my_claims = Claims {
      id: user.id,
      username: user.name.to_owned(),
      iss: hostname,
      show_nsfw: user.show_nsfw,
      theme: user.theme.to_owned(),
      default_sort_type: user.default_sort_type,
      default_listing_type: user.default_listing_type,
      lang: user.lang.to_owned(),
      avatar: user.avatar.to_owned(),
      show_avatars: user.show_avatars.to_owned(),
    };
    encode(
      &Header::default(),
      &my_claims,
      &EncodingKey::from_secret(Settings::get().jwt_secret.as_ref()),
    )
    .unwrap()
  }

  pub fn find_by_jwt(conn: &PgConnection, jwt: &str) -> Result<User_, Error> {
    let claims: Claims = Claims::decode(&jwt).expect("Invalid token").claims;
    User_::read(&conn, claims.id)
  }
}
