use crate::ReportCombinedView;
use lemmy_db_schema::{
  newtypes::{CommunityId, PaginationCursor, PostId},
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
