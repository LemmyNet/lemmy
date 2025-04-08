use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, LanguageId, PostId, TagId},
  PostFeatureType,
};
use lemmy_db_schema_file::enums::{ListingType, PostSortType};
use lemmy_db_views::structs::{CommunityView, PostPaginationCursor, PostView, VoteView};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostResponse {
  pub post_view: PostView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// TODO this should be made into a tagged enum
/// Get a post. Needs either the post id, or comment_id.
pub struct GetPost {
  #[cfg_attr(feature = "full", ts(optional))]
  pub id: Option<PostId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_id: Option<CommentId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post response.
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  /// A list of cross-posts, or other times / communities this link has been posted to.
  pub cross_posts: Vec<PostView>,
}

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
  /// DEPRECATED, use page_cursor
  #[cfg_attr(feature = "full", ts(optional))]
  pub page: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub saved_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub read_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub liked_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disliked_only: Option<bool>,
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
  pub page_cursor: Option<PostPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post list response.
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PostPaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Like a post.
pub struct CreatePostLike {
  pub post_id: PostId,
  /// Score must be -1, 0, or 1.
  pub score: i16,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit a post.
pub struct EditPost {
  pub post_id: PostId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub url: Option<String>,
  /// An optional body for the post in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub body: Option<String>,
  /// An optional alt_text, usable for image posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub alt_text: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub nsfw: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub language_id: Option<LanguageId>,
  /// Instead of fetching a thumbnail, use a custom one.
  #[cfg_attr(feature = "full", ts(optional))]
  pub custom_thumbnail: Option<String>,
  /// Time when this post should be scheduled. Null means publish immediately.
  #[cfg_attr(feature = "full", ts(optional))]
  pub scheduled_publish_time: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub tags: Option<Vec<TagId>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete a post.
pub struct DeletePost {
  pub post_id: PostId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Remove a post (only doable by mods).
pub struct RemovePost {
  pub post_id: PostId,
  pub removed: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a post as read.
pub struct MarkPostAsRead {
  pub post_id: PostId,
  pub read: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark several posts as read.
pub struct MarkManyPostsAsRead {
  pub post_ids: Vec<PostId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Hide a post from list views
pub struct HidePost {
  pub post_id: PostId,
  pub hide: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Lock a post (prevent new comments).
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Feature a post (stickies / pins to the top).
pub struct FeaturePost {
  pub post_id: PostId,
  pub featured: bool,
  pub feature_type: PostFeatureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Save / bookmark a post.
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get metadata for a given site.
pub struct GetSiteMetadata {
  pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The site metadata response.
pub struct GetSiteMetadataResponse {
  pub metadata: LinkMetadata,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Site metadata, from its opengraph tags.
pub struct LinkMetadata {
  #[serde(flatten)]
  pub opengraph_data: OpenGraphData,
  #[cfg_attr(feature = "full", ts(optional))]
  pub content_type: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Site metadata, from its opengraph tags.
pub struct OpenGraphData {
  #[cfg_attr(feature = "full", ts(optional))]
  pub title: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub(crate) image: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub embed_video_url: Option<DbUrl>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// List post likes. Admins-only.
pub struct ListPostLikes {
  pub post_id: PostId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post likes response
pub struct ListPostLikesResponse {
  pub post_likes: Vec<VoteView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Save / bookmark a post.
pub struct SubscribePost {
  pub post_id: PostId,
  pub subscribe: bool,
}
