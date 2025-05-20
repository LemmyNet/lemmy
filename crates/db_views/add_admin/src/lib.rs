use lemmy_db_schema::newtypes::PersonId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Adds an admin to a site.
pub struct AddAdmin {
  pub person_id: PersonId,
  pub added: bool,
}
