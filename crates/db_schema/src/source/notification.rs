use crate::{
  newtypes::{
    AdminAddId,
    AdminBanId,
    AdminRemoveCommunityId,
    CommentId,
    ModAddToCommunityId,
    ModBanFromCommunityId,
    ModLockCommentId,
    ModLockPostId,
    ModRemoveCommentId,
    ModRemovePostId,
    ModTransferCommunityId,
    NotificationId,
    PersonId,
    PostId,
    PrivateMessageId,
  },
  source::private_message::PrivateMessage,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use lemmy_db_schema_file::enums::NotificationType;
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
  pub kind: NotificationType,
  pub post_id: Option<PostId>,
  pub private_message_id: Option<PrivateMessageId>,
  pub admin_add_id: Option<AdminAddId>,
  pub mod_add_to_community_id: Option<ModAddToCommunityId>,
  pub admin_ban_id: Option<AdminBanId>,
  pub mod_ban_from_community_id: Option<ModBanFromCommunityId>,
  pub mod_lock_post_id: Option<ModLockPostId>,
  pub mod_remove_comment_id: Option<ModRemoveCommentId>,
  pub admin_remove_community_id: Option<AdminRemoveCommunityId>,
  pub mod_remove_post_id: Option<ModRemovePostId>,
  pub mod_lock_comment_id: Option<ModLockCommentId>,
  pub mod_transfer_community_id: Option<ModTransferCommunityId>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = notification))]
pub struct NotificationInsertForm {
  pub recipient_id: PersonId,
  pub kind: NotificationType,
  #[new(default)]
  pub comment_id: Option<CommentId>,
  #[new(default)]
  pub post_id: Option<PostId>,
  #[new(default)]
  pub private_message_id: Option<PrivateMessageId>,
  #[new(default)]
  pub admin_add_id: Option<AdminAddId>,
  #[new(default)]
  pub mod_add_to_community_id: Option<ModAddToCommunityId>,
  #[new(default)]
  pub admin_ban_id: Option<AdminBanId>,
  #[new(default)]
  pub mod_ban_from_community_id: Option<ModBanFromCommunityId>,
  #[new(default)]
  pub mod_lock_post_id: Option<ModLockPostId>,
  #[new(default)]
  pub mod_remove_comment_id: Option<ModRemoveCommentId>,
  #[new(default)]
  pub admin_remove_community_id: Option<AdminRemoveCommunityId>,
  #[new(default)]
  pub mod_remove_post_id: Option<ModRemovePostId>,
  #[new(default)]
  pub mod_lock_comment_id: Option<ModLockCommentId>,
  #[new(default)]
  pub mod_transfer_community_id: Option<ModTransferCommunityId>,
}

impl NotificationInsertForm {
  pub fn new_post(post_id: PostId, recipient_id: PersonId, kind: NotificationType) -> Self {
    Self {
      post_id: Some(post_id),
      ..Self::new(recipient_id, kind)
    }
  }
  pub fn new_comment(
    comment_id: CommentId,
    recipient_id: PersonId,
    kind: NotificationType,
  ) -> Self {
    Self {
      comment_id: Some(comment_id),
      ..Self::new(recipient_id, kind)
    }
  }
  pub fn new_private_message(private_message: &PrivateMessage) -> Self {
    Self {
      private_message_id: Some(private_message.id),
      ..Self::new(
        private_message.recipient_id,
        NotificationType::PrivateMessage,
      )
    }
  }
}
