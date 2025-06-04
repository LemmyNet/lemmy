use lemmy_db_schema::newtypes::PersonCommentMentionId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a person mention as read.
pub struct MarkPersonCommentMentionAsRead {
  pub person_comment_mention_id: PersonCommentMentionId,
  pub read: bool,
}
