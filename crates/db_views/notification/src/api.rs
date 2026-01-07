use lemmy_db_schema::newtypes::NotificationId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark a comment reply as read.
pub struct MarkNotificationAsRead {
  pub notification_id: NotificationId,
  pub read: bool,
}
