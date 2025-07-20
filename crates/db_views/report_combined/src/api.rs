use crate::{
  CommentReportView,
  CommunityReportView,
  PostReportView,
  PrivateMessageReportView,
  ReportCombinedView,
};
use lemmy_db_schema::{
  newtypes::{
    CommentId,
    CommentReportId,
    CommunityId,
    CommunityReportId,
    PaginationCursor,
    PostId,
    PostReportId,
    PrivateMessageId,
    PrivateMessageReportId,
  },
  ReportType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// List reports.
pub struct ListReports {
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// Filter the type of report.
  pub type_: Option<ReportType>,
  /// Filter by the post id. Can return either comment or post reports.
  pub post_id: Option<PostId>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  /// Only for admins: also show reports with `violates_instance_rules=false`
  pub show_community_rule_violations: Option<bool>,
  /// If true, view all your created reports. Works for non-admins/mods also.
  pub my_reports_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The post reports response.
pub struct ListReportsResponse {
  pub reports: Vec<ReportCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The comment report response.
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A community report response.
pub struct CommunityReportResponse {
  pub community_report_view: CommunityReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Report a comment.
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a report for a community.
pub struct CreateCommunityReport {
  pub community_id: CommunityId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a post report.
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get a count of the number of reports.
pub struct GetReportCount {
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for the number of reports.
pub struct GetReportCountResponse {
  pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a comment report (only doable by mods).
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a community report.
pub struct ResolveCommunityReport {
  pub report_id: CommunityReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a post report (mods only).
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a private message report.
pub struct ResolvePrivateMessageReport {
  pub report_id: PrivateMessageReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a report for a private message.
pub struct CreatePrivateMessageReport {
  pub private_message_id: PrivateMessageId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A private message report response.
pub struct PrivateMessageReportResponse {
  pub private_message_report_view: PrivateMessageReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The post report response.
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}
