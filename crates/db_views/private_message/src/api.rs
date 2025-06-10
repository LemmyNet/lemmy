use crate::PrivateMessageView;
use lemmy_db_schema::newtypes::{PersonId, PrivateMessageId};
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete a private message.
pub struct DeletePrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit a private message.
pub struct EditPrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A single private message response.
pub struct PrivateMessageResponse {
  pub private_message_view: PrivateMessageView,
}
