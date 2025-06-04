use lemmy_db_views_reports::PrivateMessageReportView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message report response.
pub struct PrivateMessageReportResponse {
  pub private_message_report_view: PrivateMessageReportView,
}
