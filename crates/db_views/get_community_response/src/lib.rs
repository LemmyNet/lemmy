use lemmy_db_schema::{newtypes::LanguageId, source::site::Site};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The community response.
pub struct GetCommunityResponse {
  pub community_view: CommunityView,
  #[cfg_attr(feature = "full", ts(optional))]
  pub site: Option<Site>,
  pub moderators: Vec<CommunityModeratorView>,
  pub discussion_languages: Vec<LanguageId>,
}
