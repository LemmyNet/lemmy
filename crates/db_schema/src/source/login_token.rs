use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::login_token;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
pub struct LoginToken {
  pub id: i32,
  #[serde(skip)]
  pub token: String,
  pub user_id: LocalUserId,
  pub published: DateTime<Utc>,
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
