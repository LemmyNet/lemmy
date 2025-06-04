use lemmy_db_schema::newtypes::LanguageId;
use lemmy_db_views_community::CommunityView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A simple community response.
pub struct CommunityResponse {
  pub community_view: CommunityView,
  pub discussion_languages: Vec<LanguageId>,
}
