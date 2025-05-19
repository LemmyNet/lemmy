use lemmy_db_schema::newtypes::{CommunityId, PersonId};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Transfer a community to a new owner.
pub struct TransferCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}
