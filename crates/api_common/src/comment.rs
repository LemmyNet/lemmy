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
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub language_id: Option<LanguageId>,
  pub form_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetComment {
  pub id: CommentId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct EditComment {
  pub comment_id: CommentId,
  pub content: Option<String>,
  pub language_id: Option<LanguageId>,
  pub form_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DistinguishComment {
  pub comment_id: CommentId,
  pub distinguished: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteComment {
  pub comment_id: CommentId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct RemoveComment {
  pub comment_id: CommentId,
  pub removed: bool,
  pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct SaveComment {
  pub comment_id: CommentId,
  pub save: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommentResponse {
  pub comment_view: CommentView,
  pub recipient_ids: Vec<LocalUserId>,
  /// An optional front end ID, to tell which is coming back  
  pub form_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CreateCommentLike {
  pub comment_id: CommentId,
  pub score: i16,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetCommentsResponse {
  pub comments: Vec<CommentView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListCommentReports {
  pub page: Option<i64>,

  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListCommentReportsResponse {
  pub comment_reports: Vec<CommentReportView>,
}
