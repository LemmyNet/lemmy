use crate::newtypes::{
  CommentReportId,
  CommunityReportId,
  PostReportId,
  PrivateMessageReportId,
  ReportCombinedId,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::report_combined;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = report_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = report_combined_keys))]
/// A combined reports table.
pub struct ReportCombined {
  pub id: ReportCombinedId,
  pub published: DateTime<Utc>,
  pub post_report_id: Option<PostReportId>,
  pub comment_report_id: Option<CommentReportId>,
  pub private_message_report_id: Option<PrivateMessageReportId>,
  pub community_report_id: Option<CommunityReportId>,
}
