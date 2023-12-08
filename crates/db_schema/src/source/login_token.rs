use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::login_token;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

/// Stores data related to a specific user login session.
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
#[cfg_attr(feature = "full", diesel(primary_key(token)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct LoginToken {
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
