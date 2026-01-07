use lemmy_diesel_utils::pagination::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ListCommunityPendingFollows {
  /// Only shows the unapproved applications
  pub unread_only: Option<bool>,
  // Only for admins, show pending follows for communities which you dont moderate
  pub all_communities: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}
