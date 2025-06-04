use lemmy_db_schema::newtypes::CommentId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Save / bookmark a comment.
pub struct SaveComment {
  pub comment_id: CommentId,
  pub save: bool,
}
