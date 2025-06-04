use lemmy_db_views_person::PersonView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a person block.
pub struct BlockPersonResponse {
  pub person_view: PersonView,
  pub blocked: bool,
}
