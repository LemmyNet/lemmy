use lemmy_db_views_reports::CommentReportView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report response.
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}
