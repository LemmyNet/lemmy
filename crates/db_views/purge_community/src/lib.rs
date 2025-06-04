use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a community from the database. This will delete all content attached to that community.
pub struct PurgeCommunity {
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}
