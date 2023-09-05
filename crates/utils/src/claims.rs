use crate::error::LemmyError;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
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
    pub fn decode(jwt: &str, jwt_secret: &str) -> Result<TokenData<Claims>, LemmyError> {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims.remove("exp");
        let key = DecodingKey::from_secret(jwt_secret.as_ref());
        Ok(decode::<Claims>(jwt, &key, &validation)?)
    }

    pub fn jwt(local_user_id: i32, jwt_secret: &str, hostname: &str) -> Result<Jwt, LemmyError> {
        let my_claims = Claims {
            sub: local_user_id,
            iss: hostname.to_string(),
            iat: Utc::now().timestamp(),
        };

        let key = EncodingKey::from_secret(jwt_secret.as_ref());
        Ok(encode(&Header::default(), &my_claims, &key)?)
    }
}
