use lemmy_db_schema::newtypes::{CommentId, PostId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// TODO this should be made into a tagged enum
/// Get a post. Needs either the post id, or comment_id.
pub struct GetPost {
  #[cfg_attr(feature = "full", ts(optional))]
  pub id: Option<PostId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_id: Option<CommentId>,
}
