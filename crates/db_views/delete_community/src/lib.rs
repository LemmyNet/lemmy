use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete your own community.
pub struct DeleteCommunity {
  pub community_id: CommunityId,
  pub deleted: bool,
}
