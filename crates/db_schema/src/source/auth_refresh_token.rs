use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::auth_refresh_token;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = auth_refresh_token))]
pub struct AuthRefreshToken {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub token: String,
  pub last_used: chrono::NaiveDateTime,
  pub last_ip: String,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = auth_refresh_token))]
pub struct AuthRefreshTokenCreateForm {
  pub local_user_id: LocalUserId,
  pub last_ip: String,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = auth_refresh_token))]
pub struct AuthRefreshTokenUpdateForm {
  pub last_used: chrono::NaiveDateTime,
  pub last_ip: String,
}
