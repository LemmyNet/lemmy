use lemmy_db_schema::newtypes::CommunityReportId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a community report.
pub struct ResolveCommunityReport {
  pub report_id: CommunityReportId,
  pub resolved: bool,
}
