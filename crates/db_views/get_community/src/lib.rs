use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// TODO make this into a tagged enum
/// Get a community. Must provide either an id, or a name.
pub struct GetCommunity {
  #[cfg_attr(feature = "full", ts(optional))]
  pub id: Option<CommunityId>,
  /// Example: star_trek , or star_trek@xyz.tld
  #[cfg_attr(feature = "full", ts(optional))]
  pub name: Option<String>,
}
