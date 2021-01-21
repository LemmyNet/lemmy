use lemmy_db_views::{
  comment_view::CommentView,
  post_report_view::PostReportView,
  post_view::PostView,
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct CreatePost {
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub nsfw: bool,
  pub community_id: i32,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct PostResponse {
  pub post_view: PostView,
}

#[derive(Deserialize)]
pub struct GetPost {
  pub id: i32,
  pub auth: Option<String>,
}

#[derive(Serialize)]
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  pub comments: Vec<CommentView>,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Deserialize, Debug)]
pub struct GetPosts {
  pub type_: String,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  pub auth: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
}

#[derive(Deserialize)]
pub struct CreatePostLike {
  pub post_id: i32,
  pub score: i16,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct EditPost {
  pub post_id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub nsfw: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct DeletePost {
  pub post_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct RemovePost {
  pub post_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct LockPost {
  pub post_id: i32,
  pub locked: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct StickyPost {
  pub post_id: i32,
  pub stickied: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct SavePost {
  pub post_id: i32,
  pub save: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostReport {
  pub post_id: i32,
  pub reason: String,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreatePostReportResponse {
  pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvePostReport {
  pub report_id: i32,
  pub resolved: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResolvePostReportResponse {
  pub report_id: i32,
  pub resolved: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community: Option<i32>,
  pub auth: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ListPostReportsResponse {
  pub posts: Vec<PostReportView>,
}
