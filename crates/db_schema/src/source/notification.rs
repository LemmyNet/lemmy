use crate::newtypes::{CommentId, NotificationId, PersonId, PostId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::NotificationTypes;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::notification;
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
  pub recipient_id: PersonId,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub read: bool,
  pub kind: NotificationTypes,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
pub struct NotificationInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub kind: NotificationTypes,
}

impl NotificationInsertForm {
  pub fn new_post(recipient_id: PersonId, post_id: PostId, kind: NotificationTypes) -> Self {
    Self {
      recipient_id,
      post_id: Some(post_id),
      kind,
      comment_id: None,
    }
  }
  pub fn new_comment(
    recipient_id: PersonId,
    comment_id: CommentId,
    kind: NotificationTypes,
  ) -> Self {
    Self {
      recipient_id,
      comment_id: Some(comment_id),
      kind,
      post_id: None,
    }
  }
}
