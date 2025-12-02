pub use lemmy_db_schema::{
  NotificationDataType,
  newtypes::NotificationId,
  source::notification::Notification,
};
pub use lemmy_db_views_notification::{
  ListNotifications,
  NotificationView,
  api::{GetUnreadCountResponse, MarkNotificationAsRead},
};
