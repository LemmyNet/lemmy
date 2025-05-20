use lemmy_db_schema::source::login_token::LoginToken;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListLoginsResponse {
  pub logins: Vec<LoginToken>,
}
