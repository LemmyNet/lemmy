use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::login_token;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Stores data related to a specific user login session.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
pub struct LoginToken {
  pub id: i32,
  /// Jwt token for this login
  #[serde(skip)]
  pub token: String,
  pub user_id: LocalUserId,
  /// Time of login
  pub published: DateTime<Utc>,
  /// IP address where login was made from, allows invalidating logins by IP address.
  /// Could be stored in truncated format, or store derived information for better privacy.
  pub ip: Option<String>,
  pub user_agent: Option<String>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
pub struct LoginTokenCreateForm {
  pub token: String,
  pub user_id: LocalUserId,
  pub ip: Option<String>,
  pub user_agent: Option<String>,
}
