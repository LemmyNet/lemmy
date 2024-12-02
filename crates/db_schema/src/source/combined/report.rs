use crate::newtypes::{CommentReportId, PostReportId, PrivateMessageReportId, ReportCombinedId};
#[cfg(feature = "full")]
use crate::schema::report_combined;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, TS, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = report_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", cursor_keys_module(name = report_combined_keys))]
/// A combined reports table.
pub struct ReportCombined {
  pub id: ReportCombinedId,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_report_id: Option<PostReportId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_report_id: Option<CommentReportId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub private_message_report_id: Option<PrivateMessageReportId>,
}
