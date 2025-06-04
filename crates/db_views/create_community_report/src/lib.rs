use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a report for a community.
pub struct CreateCommunityReport {
  pub community_id: CommunityId,
  pub reason: String,
}
