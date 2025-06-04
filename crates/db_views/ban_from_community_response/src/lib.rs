use lemmy_db_views_person::PersonView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for banning a user from a community.
pub struct BanFromCommunityResponse {
  pub person_view: PersonView,
  pub banned: bool,
}
