use lemmy_db_schema::newtypes::{CommentId, CommentReportId, CommunityId};
use lemmy_db_views::structs::CommentReportView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Report a comment.
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report response.
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a comment report (only doable by mods).
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// List comment reports.
pub struct ListCommentReports {
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_id: Option<CommentId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  #[cfg_attr(feature = "full", ts(optional))]
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report list response.
pub struct ListCommentReportsResponse {
  pub comment_reports: Vec<CommentReportView>,
}
