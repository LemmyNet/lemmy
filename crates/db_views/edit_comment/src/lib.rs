use lemmy_db_schema::newtypes::{CommentId, LanguageId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit a comment.
pub struct EditComment {
  pub comment_id: CommentId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub content: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub language_id: Option<LanguageId>,
}
