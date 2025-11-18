use crate::LocalUserView;
use lemmy_diesel_utils::pagination::PaginationCursorNew;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminListUsers {
  pub banned_only: Option<bool>,
  pub page_cursor: Option<PaginationCursorNew>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminListUsersResponse {
  pub users: Vec<LocalUserView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursorNew>,
  pub prev_page: Option<PaginationCursorNew>,
}
