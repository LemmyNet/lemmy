use lemmy_db_schema::newtypes::{CommunityId, PostId, PostReportId};
use lemmy_db_views::{
  comment_view::CommentView,
  post_report_view::PostReportView,
  post_view::PostView,
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_utils::{request::SiteMetadata, Sensitive};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePost {
  pub name: String,
  pub community_id: CommunityId,
  pub url: Option<Url>,
  pub body: Option<String>,
  pub honeypot: Option<String>,
  pub nsfw: Option<bool>,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostResponse {
  pub post_view: PostView,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPost {
  pub id: PostId,
  pub auth: Option<Sensitive>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  pub comments: Vec<CommentView>,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPosts {
  pub type_: Option<String>,
  pub sort: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub saved_only: Option<bool>,
  pub auth: Option<Sensitive>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostLike {
  pub post_id: PostId,
  pub score: i16,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditPost {
  pub post_id: PostId,
  pub name: Option<String>,
  pub url: Option<Url>,
  pub body: Option<String>,
  pub nsfw: Option<bool>,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePost {
  pub post_id: PostId,
  pub deleted: bool,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovePost {
  pub post_id: PostId,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkPostAsRead {
  pub post_id: PostId,
  pub read: bool,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StickyPost {
  pub post_id: PostId,
  pub stickied: bool,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPostReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  pub auth: Sensitive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPostReportsResponse {
  pub post_reports: Vec<PostReportView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSiteMetadata {
  pub url: Url,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetSiteMetadataResponse {
  pub metadata: SiteMetadata,
}
