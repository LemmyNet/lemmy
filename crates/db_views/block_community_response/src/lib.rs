use lemmy_db_views_community::CommunityView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The block community response.
pub struct BlockCommunityResponse {
  pub community_view: CommunityView,
  pub blocked: bool,
}
