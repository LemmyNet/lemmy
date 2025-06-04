use lemmy_db_schema::newtypes::{CommentId, LanguageId, PostId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a comment.
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub parent_id: Option<CommentId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub language_id: Option<LanguageId>,
}
