use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentId, CommentReportId, CommunityId, LocalUserId, PostId},
  CommentSortType,
  ListingType,
};
use lemmy_db_views::structs::{CommentReportView, CommentView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub form_id: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetComment {
  pub id: CommentId,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditComment {
  pub content: String,
  pub comment_id: CommentId,
  pub form_id: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeleteComment {
  pub comment_id: CommentId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RemoveComment {
  pub comment_id: CommentId,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SaveComment {
  pub comment_id: CommentId,
  pub save: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentResponse {
  pub comment_view: CommentView,
  pub recipient_ids: Vec<LocalUserId>,
  pub form_id: Option<String>, // An optional front end ID, to tell which is coming back
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreateCommentLike {
  pub comment_id: CommentId,
  pub score: i16,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetComments {
  pub type_: Option<ListingType>,
  pub sort: Option<CommentSortType>,
  pub max_depth: Option<i32>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub post_id: Option<PostId>,
  pub parent_id: Option<CommentId>,
  pub saved_only: Option<bool>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetCommentsResponse {
  pub comments: Vec<CommentView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListCommentReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListCommentReportsResponse {
  pub comment_reports: Vec<CommentReportView>,
}
