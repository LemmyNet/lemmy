use lemmy_db_schema::newtypes::CommentId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Like a comment.
pub struct CreateCommentLike {
  pub comment_id: CommentId,
  /// Must be -1, 0, or 1 .
  pub score: i16,
}
