use lemmy_db_schema::LocalUserSortType;
use lemmy_diesel_utils::pagination::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminListUsers {
  pub banned_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub sort: Option<LocalUserSortType>,
  pub limit: Option<i64>,
}
