use crate::PostView;
use lemmy_db_schema::{
  PostFeatureType,
  newtypes::{CommunityId, CommunityTagId, LanguageId, MultiCommunityId, PostId},
};
use lemmy_db_schema_file::enums::{ListingType, PostNotificationsMode, PostSortType};
use lemmy_diesel_utils::{dburl::DbUrl, pagination::PaginationCursor};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a post.
pub struct CreatePost {
  pub name: String,
  pub community_id: CommunityId,
  pub url: Option<String>,
  /// An optional body for the post in markdown.
  pub body: Option<String>,
  /// An optional alt_text, usable for image posts.
  pub alt_text: Option<String>,
  /// A honeypot to catch bots. Should be None.
  pub honeypot: Option<String>,
  pub nsfw: Option<bool>,
  pub language_id: Option<LanguageId>,
  /// Instead of fetching a thumbnail, use a custom one.
  pub custom_thumbnail: Option<String>,
  pub tags: Option<Vec<CommunityTagId>>,
  /// Time when this post should be scheduled. Null means publish immediately.
  pub scheduled_publish_time_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Like a post.
pub struct CreatePostLike {
  pub post_id: PostId,
  /// True means Upvote, False means Downvote, and None means remove vote.
  pub is_upvote: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete a post.
pub struct DeletePost {
  pub post_id: PostId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edit a post.
pub struct EditPost {
  pub post_id: PostId,
  pub name: Option<String>,
  pub url: Option<String>,
  /// An optional body for the post in markdown.
  pub body: Option<String>,
  /// An optional alt_text, usable for image posts.
  pub alt_text: Option<String>,
  pub nsfw: Option<bool>,
  pub language_id: Option<LanguageId>,
  /// Instead of fetching a thumbnail, use a custom one.
  pub custom_thumbnail: Option<String>,
  /// Time when this post should be scheduled. Null means publish immediately.
  pub scheduled_publish_time_at: Option<i64>,
  pub tags: Option<Vec<CommunityTagId>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mods can change some metadata for posts
pub struct ModEditPost {
  pub post_id: PostId,
  pub nsfw: Option<bool>,
  pub tags: Option<Vec<CommunityTagId>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Feature a post (stickies / pins to the top).
pub struct FeaturePost {
  pub post_id: PostId,
  pub featured: bool,
  pub feature_type: PostFeatureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Change notification settings for a post
pub struct EditPostNotifications {
  pub post_id: PostId,
  pub mode: PostNotificationsMode,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get a list of posts.
pub struct GetPosts {
  pub type_: Option<ListingType>,
  pub sort: Option<PostSortType>,
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  /// Use Zero to override the local_site and local_user time_range.
  pub time_range_seconds: Option<i32>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub multi_community_id: Option<MultiCommunityId>,
  pub multi_community_name: Option<String>,
  pub show_hidden: Option<bool>,
  /// If true, then show the read posts (even if your user setting is to hide them)
  pub show_read: Option<bool>,
  /// If true, then show the nsfw posts (even if your user setting is to hide them)
  pub show_nsfw: Option<bool>,
  /// If false, then show posts with media attached (even if your user setting is to hide them)
  pub hide_media: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  pub mark_as_read: Option<bool>,
  /// If true, then only show posts with no comments
  pub no_comments_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  /// For backwards compat with API v3 (not available on API v4)
  #[serde(skip)]
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get metadata for a given site.
pub struct GetSiteMetadata {
  pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The site metadata response.
pub struct GetSiteMetadataResponse {
  pub metadata: LinkMetadata,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Site metadata, from its opengraph tags.
pub struct LinkMetadata {
  #[serde(flatten)]
  pub opengraph_data: OpenGraphData,
  pub content_type: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Hide a post from list views
pub struct HidePost {
  pub post_id: PostId,
  pub hide: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// List post likes. Admins-only.
pub struct ListPostLikes {
  pub post_id: PostId,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Lock a post (prevent new comments).
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark a post as read.
pub struct MarkPostAsRead {
  pub post_id: PostId,
  pub read: bool,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Site metadata, from its opengraph tags.
pub struct OpenGraphData {
  pub title: Option<String>,
  pub description: Option<String>,
  pub image: Option<DbUrl>,
  pub image_width: Option<u16>,
  pub image_height: Option<u16>,
  pub embed_video_url: Option<DbUrl>,
  pub video_width: Option<u16>,
  pub video_height: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PostResponse {
  pub post_view: PostView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Purges a post from the database. This will delete all content attached to that post.
pub struct PurgePost {
  pub post_id: PostId,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Remove a post (only doable by mods).
pub struct RemovePost {
  pub post_id: PostId,
  pub removed: bool,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Save / bookmark a post.
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Mark several posts as read.
pub struct MarkManyPostsAsRead {
  pub post_ids: Vec<PostId>,
  pub read: bool,
}
