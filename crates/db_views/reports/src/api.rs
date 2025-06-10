use crate::{CommentReportView, CommunityReportView, PostReportView, PrivateMessageReportView};
use lemmy_db_schema::newtypes::{
  CommentId,
  CommentReportId,
  CommunityId,
  CommunityReportId,
  PostId,
  PostReportId,
  PrivateMessageId,
  PrivateMessageReportId,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report response.
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A community report response.
pub struct CommunityReportResponse {
  pub community_report_view: CommunityReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Report a comment.
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub violates_instance_rules: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a report for a community.
pub struct CreateCommunityReport {
  pub community_id: CommunityId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a post report.
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub violates_instance_rules: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get a count of the number of reports.
pub struct GetReportCount {
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for the number of reports.
pub struct GetReportCountResponse {
  pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a comment report (only doable by mods).
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a community report.
pub struct ResolveCommunityReport {
  pub report_id: CommunityReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a post report (mods only).
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Resolve a private message report.
pub struct ResolvePrivateMessageReport {
  pub report_id: PrivateMessageReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a report for a private message.
pub struct CreatePrivateMessageReport {
  pub private_message_id: PrivateMessageId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message report response.
pub struct PrivateMessageReportResponse {
  pub private_message_report_view: PrivateMessageReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post report response.
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}
