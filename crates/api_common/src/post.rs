use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, PostId, PostReportId},
  ListingType,
  SortType,
};
use lemmy_db_views::structs::{PostReportView, PostView};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePost {
  pub name: String,
  pub community_id: CommunityId,
  pub url: Option<Url>,
  pub body: Option<String>,
  pub honeypot: Option<String>,
  pub nsfw: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostResponse {
  pub post_view: PostView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPost {
  pub id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetPosts {
  pub type_: Option<ListingType>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub saved_only: Option<bool>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePostLike {
  pub post_id: PostId,
  pub score: i16,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditPost {
  pub post_id: PostId,
  pub name: Option<String>,
  pub url: Option<Url>,
  pub body: Option<String>,
  pub nsfw: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeletePost {
  pub post_id: PostId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RemovePost {
  pub post_id: PostId,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkPostAsRead {
  pub post_id: PostId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StickyPost {
  pub post_id: PostId,
  pub stickied: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListPostReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListPostReportsResponse {
  pub post_reports: Vec<PostReportView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSiteMetadata {
  pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSiteMetadataResponse {
  pub metadata: SiteMetadata,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct SiteMetadata {
  pub title: Option<String>,
  pub description: Option<String>,
  pub(crate) image: Option<DbUrl>,
  pub embed_video_url: Option<DbUrl>,
}
