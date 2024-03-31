use serde::{Deserialize, Serialize};
use lemmy_db_views::structs::TaglineView;
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for custom emojis.
pub struct ListTaglinesResponse {
  pub taglines: Vec<TaglineView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches a list of registration applications.
pub struct ListTaglines {
  pub page: Option<i64>,
  pub limit: Option<i64>,
}
