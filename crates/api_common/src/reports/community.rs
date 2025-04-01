use lemmy_db_schema::newtypes::{CommunityId, CommunityReportId};
use lemmy_db_views::structs::CommunityReportView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a report for a community.
pub struct CreateCommunityReport {
  pub community_id: CommunityId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A community report response.
pub struct CommunityReportResponse {
  pub community_report_view: CommunityReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a community report.
pub struct ResolveCommunityReport {
  pub report_id: CommunityReportId,
  pub resolved: bool,
}
