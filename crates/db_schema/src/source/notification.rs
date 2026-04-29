use crate::{
  newtypes::{CommentId, CommunityId, ModlogId, NotificationId, PostId, PrivateMessageId},
  source::{comment::Comment, modlog::Modlog, post::Post, private_message::PrivateMessage},
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::notification;
use lemmy_db_schema_file::{InstanceId, PersonId, enums::NotificationType};
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
  pub kind: NotificationType,
  pub post_id: Option<PostId>,
  pub private_message_id: Option<PrivateMessageId>,
  pub modlog_id: Option<ModlogId>,
  pub creator_id: PersonId,
  #[serde(skip)]
  pub instance_id: Option<InstanceId>,
  #[serde(skip)]
  pub community_id: Option<CommunityId>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
pub struct NotificationInsertForm {
  pub recipient_id: PersonId,
  pub creator_id: PersonId,
  pub kind: NotificationType,
  #[new(default)]
  pub comment_id: Option<CommentId>,
  #[new(default)]
  pub post_id: Option<PostId>,
  #[new(default)]
  pub private_message_id: Option<PrivateMessageId>,
  #[new(default)]
  pub modlog_id: Option<ModlogId>,
  #[new(default)]
  pub instance_id: Option<InstanceId>,
  #[new(default)]
  pub community_id: Option<CommunityId>,
}

impl NotificationInsertForm {
  pub fn new_post(post: &Post, recipient_id: PersonId, kind: NotificationType) -> Self {
    Self {
      post_id: Some(post.id),
      community_id: Some(post.community_id),
      ..Self::new(recipient_id, post.creator_id, kind)
    }
  }
  pub fn new_comment(comment: &Comment, recipient_id: PersonId, kind: NotificationType) -> Self {
    Self {
      comment_id: Some(comment.id),
      post_id: Some(comment.post_id),
      community_id: Some(comment.community_id),
      ..Self::new(recipient_id, comment.creator_id, kind)
    }
  }
  pub fn new_private_message(private_message: &PrivateMessage) -> Self {
    Self {
      private_message_id: Some(private_message.id),
      ..Self::new(
        private_message.recipient_id,
        private_message.creator_id,
        NotificationType::PrivateMessage,
      )
    }
  }

  pub fn new_mod_action(action: &Modlog, recipient_id: PersonId) -> Self {
    Self {
      modlog_id: Some(action.id),
      comment_id: action.target_comment_id,
      post_id: action.target_post_id,
      community_id: action.target_community_id,
      instance_id: action.target_instance_id,
      ..Self::new(recipient_id, action.mod_id, NotificationType::ModAction)
    }
  }
}
