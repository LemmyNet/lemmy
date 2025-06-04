use lemmy_db_schema::newtypes::PersonId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a private message.
pub struct CreatePrivateMessage {
  pub content: String,
  pub recipient_id: PersonId,
}
