use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentId, CommentReportId, CommunityId, LanguageId, LocalUserId, PostId},
  CommentSortType,
  ListingType,
};
use lemmy_db_views::structs::{CommentReportView, CommentView};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a comment.
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub language_id: Option<LanguageId>,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetch an individual comment.
pub struct GetComment {
  pub id: CommentId,
  pub auth: Option<Sensitive<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit a comment.
pub struct EditComment {
  pub comment_id: CommentId,
  pub content: Option<String>,
  pub language_id: Option<LanguageId>,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Distinguish a comment (IE speak as moderator).
pub struct DistinguishComment {
  pub comment_id: CommentId,
  pub distinguished: bool,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete your own comment.
pub struct DeleteComment {
  pub comment_id: CommentId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Remove a comment (only doable by mods).
pub struct RemoveComment {
  pub comment_id: CommentId,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Save / bookmark a comment.
pub struct SaveComment {
  pub comment_id: CommentId,
  pub save: bool,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment response.
pub struct CommentResponse {
  pub comment_view: CommentView,
  pub recipient_ids: Vec<LocalUserId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Like a comment.
pub struct CreateCommentLike {
  pub comment_id: CommentId,
  /// Must be -1, 0, or 1 .
  pub score: i16,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get a list of comments.
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
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment list response.
pub struct GetCommentsResponse {
  pub comments: Vec<CommentView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Report a comment.
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report response.
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a comment report (only doable by mods).
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// List comment reports.
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
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report list response.
pub struct ListCommentReportsResponse {
  pub comment_reports: Vec<CommentReportView>,
}
