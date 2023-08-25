use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, LanguageId, PostId, PostReportId},
  ListingType,
  PostFeatureType,
  SortType,
};
use lemmy_db_views::structs::{PostReportView, PostView};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_proc_macros::lemmy_dto;
use url::Url;

#[lemmy_dto(Default)]
/// Create a post.
pub struct CreatePost {
  pub name: String,
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub url: Option<Url>,
  /// An optional body for the post in markdown.
  pub body: Option<String>,
  /// A honeypot to catch bots. Should be None.
  pub honeypot: Option<String>,
  pub nsfw: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
pub struct PostResponse {
  pub post_view: PostView,
}

#[lemmy_dto]
/// Get a post. Needs either the post id, or comment_id.
pub struct GetPost {
  pub id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto]
/// The post response.
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  /// A list of cross-posts, or other times / communities this link has been posted to.
  pub cross_posts: Vec<PostView>,
}

#[lemmy_dto(Default)]
/// Get a list of posts.
pub struct GetPosts {
  pub type_: Option<ListingType>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub saved_only: Option<bool>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  pub moderator_view: Option<bool>,
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto]
/// The post list response.
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
}

#[lemmy_dto(Default)]
/// Like a post.
pub struct CreatePostLike {
  pub post_id: PostId,
  /// Score must be -1, 0, or 1.
  pub score: i16,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Edit a post.
pub struct EditPost {
  pub post_id: PostId,
  pub name: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub url: Option<Url>,
  /// An optional body for the post in markdown.
  pub body: Option<String>,
  pub nsfw: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Delete a post.
pub struct DeletePost {
  pub post_id: PostId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Remove a post (only doable by mods).
pub struct RemovePost {
  pub post_id: PostId,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Mark a post as read.
pub struct MarkPostAsRead {
  pub post_id: PostId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Lock a post (prevent new comments).
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Feature a post (stickies / pins to the top).
pub struct FeaturePost {
  pub post_id: PostId,
  pub featured: bool,
  pub feature_type: PostFeatureType,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Save / bookmark a post.
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Create a post report.
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The post report response.
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}

#[lemmy_dto(Default)]
/// Resolve a post report (mods only).
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// List post reports.
pub struct ListPostReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The post reports response.
pub struct ListPostReportsResponse {
  pub post_reports: Vec<PostReportView>,
}

#[lemmy_dto]
/// Get metadata for a given site.
pub struct GetSiteMetadata {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub url: Url,
}

#[lemmy_dto]
/// The site metadata response.
pub struct GetSiteMetadataResponse {
  pub metadata: SiteMetadata,
}

#[lemmy_dto(PartialEq, Eq)]
/// Site metadata, from its opengraph tags.
pub struct SiteMetadata {
  pub title: Option<String>,
  pub description: Option<String>,
  pub(crate) image: Option<DbUrl>,
  pub embed_video_url: Option<DbUrl>,
}
