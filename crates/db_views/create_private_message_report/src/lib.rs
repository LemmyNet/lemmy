use lemmy_db_schema::newtypes::PrivateMessageId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a report for a private message.
pub struct CreatePrivateMessageReport {
  pub private_message_id: PrivateMessageId,
  pub reason: String,
}
