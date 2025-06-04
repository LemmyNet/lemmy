use lemmy_db_schema::newtypes::CommentId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Remove a comment (only doable by mods).
pub struct RemoveComment {
  pub comment_id: CommentId,
  pub removed: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}
