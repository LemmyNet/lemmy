use lemmy_db_schema::newtypes::PostId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Lock a post (prevent new comments).
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}
