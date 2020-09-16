use lemmy_db::comment_view::CommentView;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateComment {
  pub content: String,
  pub parent_id: Option<i32>,
  pub post_id: i32,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct EditComment {
  pub content: String,
  pub edit_id: i32,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct DeleteComment {
  pub edit_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct RemoveComment {
  pub edit_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct MarkCommentAsRead {
  pub edit_id: i32,
  pub read: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct SaveComment {
  pub comment_id: i32,
  pub save: bool,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct CommentResponse {
  pub comment: CommentView,
  pub recipient_ids: Vec<i32>,
  pub form_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCommentLike {
  pub comment_id: i32,
  pub score: i16,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct GetComments {
  pub type_: String,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  pub auth: Option<String>,
}

#[derive(Serialize)]
pub struct GetCommentsResponse {
  pub comments: Vec<CommentView>,
}
