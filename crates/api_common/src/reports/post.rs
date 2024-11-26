use lemmy_db_schema::newtypes::{CommunityId, PostId, PostReportId};
use lemmy_db_views::structs::PostReportView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a post report.
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post report response.
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a post report (mods only).
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// List post reports.
pub struct ListPostReports {
  #[cfg_attr(feature = "full", ts(optional))]
  pub page: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  #[cfg_attr(feature = "full", ts(optional))]
  pub unresolved_only: Option<bool>,
  // TODO make into tagged enum at some point
  /// if no community is given, it returns reports for all communities moderated by the auth user
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_id: Option<PostId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post reports response.
pub struct ListPostReportsResponse {
  pub post_reports: Vec<PostReportView>,
}
