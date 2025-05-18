use lemmy_db_schema::newtypes::TagId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Update a community tag.
pub struct UpdateCommunityTag {
  pub tag_id: TagId,
  pub display_name: String,
}
