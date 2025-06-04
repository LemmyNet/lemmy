use lemmy_db_schema::newtypes::TaglineId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Update a tagline
pub struct UpdateTagline {
  pub id: TaglineId,
  pub content: String,
}
