use lemmy_db_schema_file::enums::ListingType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches a random community
pub struct GetRandomCommunity {
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<ListingType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
}
