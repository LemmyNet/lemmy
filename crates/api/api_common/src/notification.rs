pub use lemmy_db_schema::{
  newtypes::NotificationId,
  source::notification::Notification,
  NotificationDataType,
};
pub use lemmy_db_views_notification::{
  api::{GetUnreadCountResponse, MarkNotificationAsRead, MarkPrivateMessageAsRead},
  ListNotifications,
  ListNotificationsResponse,
  NotificationView,
};
