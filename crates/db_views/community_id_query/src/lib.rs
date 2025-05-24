use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

/// Parameter for setting community icon or banner. Can't use POST data here as it already contains
/// the image data.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommunityIdQuery {
  pub id: CommunityId,
}
