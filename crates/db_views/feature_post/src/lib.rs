use lemmy_db_schema::{newtypes::PostId, PostFeatureType};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Feature a post (stickies / pins to the top).
pub struct FeaturePost {
  pub post_id: PostId,
  pub featured: bool,
  pub feature_type: PostFeatureType,
}
