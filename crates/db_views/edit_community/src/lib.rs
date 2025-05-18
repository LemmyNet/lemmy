use lemmy_db_schema::newtypes::{CommunityId, LanguageId};
use lemmy_db_schema_file::enums::CommunityVisibility;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit a community.
pub struct EditCommunity {
  pub community_id: CommunityId,
  /// A longer title.
  #[cfg_attr(feature = "full", ts(optional))]
  pub title: Option<String>,
  /// A sidebar for the community in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub sidebar: Option<String>,
  /// A shorter, one line description of your community.
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  /// Whether its an NSFW community.
  #[cfg_attr(feature = "full", ts(optional))]
  pub nsfw: Option<bool>,
  /// Whether to restrict posting only to moderators.
  #[cfg_attr(feature = "full", ts(optional))]
  pub posting_restricted_to_mods: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub discussion_languages: Option<Vec<LanguageId>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub visibility: Option<CommunityVisibility>,
}