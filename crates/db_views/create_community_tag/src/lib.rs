use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a tag for a community.
pub struct CreateCommunityTag {
  pub community_id: CommunityId,
  pub display_name: String,
}
