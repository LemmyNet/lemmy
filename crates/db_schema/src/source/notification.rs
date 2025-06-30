use crate::newtypes::{CommentId, LocalUserId, NotificationId, PostId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::NotificationTypes;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{local_user_notification, notification};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person mention.
pub struct Notification {
  pub id: NotificationId,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
pub struct NotificationInsertForm {
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_notification))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct LocalUserNotification {
  pub notification_id: NotificationId,
  pub recipient_id: LocalUserId,
  pub kind: NotificationTypes,
  pub read: bool,
}

#[cfg_attr(feature = "full", derive(Insertable, derive_new::new))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_notification))]
pub struct LocalUserNotificationInsertForm {
  pub notification_id: NotificationId,
  pub recipient_id: LocalUserId,
  pub kind: NotificationTypes,
}

impl NotificationInsertForm {
  pub fn new_post(post_id: PostId) -> Self {
    Self {
      post_id: Some(post_id),
      comment_id: None,
    }
  }
  pub fn new_comment(comment_id: CommentId) -> Self {
    Self {
      comment_id: Some(comment_id),
      post_id: None,
    }
  }
}
