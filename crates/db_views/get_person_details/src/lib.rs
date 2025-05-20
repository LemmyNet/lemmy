use lemmy_db_schema::newtypes::PersonId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets a person's details.
///
/// Either person_id, or username are required.
pub struct GetPersonDetails {
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  #[cfg_attr(feature = "full", ts(optional))]
  pub username: Option<String>,
}
