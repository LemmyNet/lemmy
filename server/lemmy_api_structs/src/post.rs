use lemmy_db::{
  comment_view::CommentView,
  community_view::{CommunityModeratorView, CommunityView},
  post_view::PostView,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePost {
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub nsfw: bool,
  pub community_id: i32,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostResponse {
  pub post: PostView,
}

#[derive(Serialize, Deserialize)]
pub struct GetPost {
  pub id: i32,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPostResponse {
  pub post: PostView,
  pub comments: Vec<CommentView>,
  pub community: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPosts {
  pub type_: String,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostLike {
  pub post_id: i32,
  pub score: i16,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditPost {
  pub edit_id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub nsfw: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeletePost {
  pub edit_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemovePost {
  pub edit_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct LockPost {
  pub edit_id: i32,
  pub locked: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct StickyPost {
  pub edit_id: i32,
  pub stickied: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct SavePost {
  pub post_id: i32,
  pub save: bool,
  pub auth: String,
}
