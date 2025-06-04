use lemmy_db_schema::newtypes::LanguageId;
use lemmy_db_schema_file::enums::CommunityVisibility;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
/// Create a community.
pub struct CreateCommunity {
  /// The unique name.
  pub name: String,
  /// A longer title.
  pub title: String,
  /// A sidebar for the community in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub sidebar: Option<String>,
  /// A shorter, one line description of your community.
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  /// An icon URL.
  #[cfg_attr(feature = "full", ts(optional))]
  pub icon: Option<String>,
  /// A banner URL.
  #[cfg_attr(feature = "full", ts(optional))]
  pub banner: Option<String>,
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
