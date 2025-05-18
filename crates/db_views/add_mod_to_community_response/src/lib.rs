use lemmy_db_views_community_moderator::CommunityModeratorView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of adding a moderator to a community.
pub struct AddModToCommunityResponse {
  pub moderators: Vec<CommunityModeratorView>,
}
