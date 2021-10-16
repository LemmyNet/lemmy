use lemmy_db_schema::newtypes::{CommentId, CommentReportId, CommunityId, LocalUserId, PostId};
use lemmy_db_views::{comment_report_view::CommentReportView, comment_view::CommentView};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditComment {
  pub content: String,
  pub comment_id: CommentId,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteComment {
  pub comment_id: CommentId,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveComment {
  pub comment_id: CommentId,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct MarkCommentAsRead {
  pub comment_id: CommentId,
  pub read: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct SaveComment {
  pub comment_id: CommentId,
  pub save: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentResponse {
  pub comment_view: CommentView,
  pub recipient_ids: Vec<LocalUserId>,
  pub form_id: Option<String>, // An optional front end ID, to tell which is coming back
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLike {
  pub comment_id: CommentId,
  pub score: i16,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetComments {
  pub type_: Option<String>,
  pub sort: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub saved_only: Option<bool>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommentsResponse {
  pub comments: Vec<CommentView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Serialize, Deserialize)]
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListCommentReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListCommentReportsResponse {
  pub comment_reports: Vec<CommentReportView>,
}
