use lemmy_db_schema::newtypes::PostId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a post from the database. This will delete all content attached to that post.
pub struct PurgePost {
  pub post_id: PostId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}
