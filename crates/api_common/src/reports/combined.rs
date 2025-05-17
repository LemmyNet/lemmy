use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_views::structs::{ReportCombinedPaginationCursor, ReportCombinedView};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// List reports.
pub struct ListReports {
  /// Only shows the unresolved reports
  #[cfg_attr(feature = "full", ts(optional))]
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<ReportCombinedPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post reports response.
pub struct ListReportsResponse {
  pub reports: Vec<ReportCombinedView>,
}
