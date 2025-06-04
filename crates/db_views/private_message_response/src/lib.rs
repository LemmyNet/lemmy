use lemmy_db_views_private_message::PrivateMessageView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A single private message response.
pub struct PrivateMessageResponse {
  pub private_message_view: PrivateMessageView,
}
