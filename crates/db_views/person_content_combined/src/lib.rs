use lemmy_db_schema::PersonContentType;
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::pagination::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets a person's content (posts and comments)
///
/// Either person_id, or username are required.
pub struct ListPersonContent {
  pub type_: Option<PersonContentType>,
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}
