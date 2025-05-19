use lemmy_db_schema::newtypes::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListCommunityPendingFollows {
  /// Only shows the unapproved applications
  #[cfg_attr(feature = "full", ts(optional))]
  pub pending_only: Option<bool>,
  // Only for admins, show pending follows for communities which you dont moderate
  #[cfg_attr(feature = "full", ts(optional))]
  pub all_communities: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}
