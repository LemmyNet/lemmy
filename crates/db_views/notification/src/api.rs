use lemmy_db_schema::newtypes::NotificationId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response containing a count of unread notifications.
pub struct GetUnreadCountResponse {
  pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The count of unread registration applications.
pub struct GetUnreadRegistrationApplicationCountResponse {
  pub registration_applications: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark a comment reply as read.
pub struct MarkNotificationAsRead {
  pub notification_id: NotificationId,
  pub read: bool,
}
