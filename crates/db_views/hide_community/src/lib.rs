use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Hide a community from the main view.
pub struct HideCommunity {
  pub community_id: CommunityId,
  pub hidden: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}
