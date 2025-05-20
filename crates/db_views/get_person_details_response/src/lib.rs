use lemmy_db_schema::source::site::Site;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_person::PersonView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A person's details response.
pub struct GetPersonDetailsResponse {
  pub person_view: PersonView,
  #[cfg_attr(feature = "full", ts(optional))]
  pub site: Option<Site>,
  pub moderates: Vec<CommunityModeratorView>,
}
