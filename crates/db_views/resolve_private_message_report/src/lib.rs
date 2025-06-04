use lemmy_db_schema::newtypes::PrivateMessageReportId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a private message report.
pub struct ResolvePrivateMessageReport {
  pub report_id: PrivateMessageReportId,
  pub resolved: bool,
}
