use crate::newtypes::{CommentId, NotificationId, PersonId, PostId, PrivateMessageId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use lemmy_db_schema_file::enums::NotificationTypes;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::notification;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[cfg_attr(feature = "full", cursor_keys_module(name = notification_keys))]
pub struct Notification {
  pub id: NotificationId,
  pub recipient_id: PersonId,
  pub comment_id: Option<CommentId>,
  pub read: bool,
  pub published_at: DateTime<Utc>,
  pub kind: NotificationTypes,
  pub post_id: Option<PostId>,
  pub private_message_id: Option<PrivateMessageId>,
}

#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
pub struct NotificationInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: Option<CommentId>,
  pub kind: NotificationTypes,
  pub post_id: Option<PostId>,
  pub private_message_id: Option<PrivateMessageId>,
}

impl NotificationInsertForm {
  pub fn new_post(post_id: PostId, recipient_id: PersonId, kind: NotificationTypes) -> Self {
    Self {
      post_id: Some(post_id),
      comment_id: None,
      private_message_id: None,
      recipient_id,
      kind,
    }
  }
  pub fn new_comment(
    comment_id: CommentId,
    recipient_id: PersonId,
    kind: NotificationTypes,
  ) -> Self {
    Self {
      post_id: None,
      comment_id: Some(comment_id),
      private_message_id: None,
      recipient_id,
      kind,
    }
  }
  pub fn new_private_message(
    private_message_id: PrivateMessageId,
    recipient_id: PersonId,
    kind: NotificationTypes,
  ) -> Self {
    Self {
      post_id: None,
      comment_id: None,
      private_message_id: Some(private_message_id),
      recipient_id,
      kind,
    }
  }
}
