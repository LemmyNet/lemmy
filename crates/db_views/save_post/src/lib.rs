use lemmy_db_schema::newtypes::PostId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Save / bookmark a post.
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
}
