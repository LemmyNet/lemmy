use lemmy_db_schema::newtypes::{CommunityId, LanguageId, TagId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a post.
pub struct CreatePost {
  pub name: String,
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub url: Option<String>,
  /// An optional body for the post in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub body: Option<String>,
  /// An optional alt_text, usable for image posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub alt_text: Option<String>,
  /// A honeypot to catch bots. Should be None.
  #[cfg_attr(feature = "full", ts(optional))]
  pub honeypot: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub nsfw: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub language_id: Option<LanguageId>,
  /// Instead of fetching a thumbnail, use a custom one.
  #[cfg_attr(feature = "full", ts(optional))]
  pub custom_thumbnail: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub tags: Option<Vec<TagId>>,
  /// Time when this post should be scheduled. Null means publish immediately.
  #[cfg_attr(feature = "full", ts(optional))]
  pub scheduled_publish_time: Option<i64>,
}
