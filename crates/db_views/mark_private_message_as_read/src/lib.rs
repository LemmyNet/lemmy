use lemmy_db_schema::newtypes::PrivateMessageId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a private message as read.
pub struct MarkPrivateMessageAsRead {
  pub private_message_id: PrivateMessageId,
  pub read: bool,
}
