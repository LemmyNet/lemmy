use lemmy_db_schema::PersonContentType;
#[cfg(feature = "full")]
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::pagination::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
pub struct ListPersonSaved {
  pub type_: Option<PersonContentType>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}
