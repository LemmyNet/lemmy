use lemmy_db_schema::newtypes::{CommunityId, PersonId};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ApproveCommunityPendingFollower {
  pub community_id: CommunityId,
  pub follower_id: PersonId,
  pub approve: bool,
}
