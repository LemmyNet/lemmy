use lemmy_db_schema::newtypes::CommentId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Report a comment.
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub violates_instance_rules: Option<bool>,
}
