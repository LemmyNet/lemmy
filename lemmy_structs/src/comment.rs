use lemmy_db_views::{comment_report_view::CommentReportView, comment_view::CommentView};
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
  pub comment_id: i32,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct DeleteComment {
  pub comment_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct RemoveComment {
  pub comment_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct MarkCommentAsRead {
  pub comment_id: i32,
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
  pub comment_view: CommentView,
  pub recipient_ids: Vec<i32>, // TODO another way to do this? Maybe a UserMention belongs to Comment
  pub form_id: Option<String>, // An optional front end ID, to tell which is coming back
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

#[derive(Serialize, Deserialize)]
pub struct CreateCommentReport {
  pub comment_id: i32,
  pub reason: String,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateCommentReportResponse {
  pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveCommentReport {
  pub report_id: i32,
  pub resolved: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResolveCommentReportResponse {
  // TODO this should probably return the view
  pub report_id: i32,
  pub resolved: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommentReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community: Option<i32>,
  pub auth: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ListCommentReportsResponse {
  pub comments: Vec<CommentReportView>,
}
