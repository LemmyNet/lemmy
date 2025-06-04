use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminBlockInstanceParams {
  pub instance: String,
  pub block: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<DateTime<Utc>>,
}
