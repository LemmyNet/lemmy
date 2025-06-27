pub use lemmy_db_schema::{
  newtypes::NotificationId,
  source::notification::Notification,
  InboxDataType,
};
pub use lemmy_db_views_inbox_combined::{
  api::{GetUnreadCountResponse, MarkNotificationAsRead, MarkPrivateMessageAsRead},
  InboxCombinedView,
  ListInbox,
  ListInboxResponse,
  NotificationView,
};
