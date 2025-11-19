use lemmy_db_views_post::PostView;
use lemmy_diesel_utils::pagination::PaginationCursorNew;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your hidden posts.
pub struct ListPersonHidden {
  pub page_cursor: Option<PaginationCursorNew>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your read posts.
pub struct ListPersonRead {
  pub page_cursor: Option<PaginationCursorNew>,
  pub limit: Option<i64>,
}
