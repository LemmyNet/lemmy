use lemmy_db_schema::newtypes::{CommunityId, PaginationCursor};
use lemmy_db_schema_file::enums::{ListingType, PostSortType};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get a list of posts.
pub struct GetPosts {
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<ListingType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub sort: Option<PostSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  /// Use Zero to override the local_site and local_user time_range.
  pub time_range_seconds: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_hidden: Option<bool>,
  /// If true, then show the read posts (even if your user setting is to hide them)
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_read: Option<bool>,
  /// If true, then show the nsfw posts (even if your user setting is to hide them)
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// If false, then show posts with media attached (even if your user setting is to hide them)
  #[cfg_attr(feature = "full", ts(optional))]
  pub hide_media: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  #[cfg_attr(feature = "full", ts(optional))]
  pub mark_as_read: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// If true, then only show posts with no comments
  pub no_comments_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}
