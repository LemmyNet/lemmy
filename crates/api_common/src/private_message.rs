use crate::sensitive::Sensitive;
use lemmy_db_schema::newtypes::{PersonId, PrivateMessageId, PrivateMessageReportId};
use lemmy_db_views::structs::{PrivateMessageReportView, PrivateMessageView};
use lemmy_proc_macros::lemmy_dto;

#[lemmy_dto(Default)]
/// Create a private message.
pub struct CreatePrivateMessage {
  pub content: String,
  pub recipient_id: PersonId,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Edit a private message.
pub struct EditPrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub content: String,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Delete a private message.
pub struct DeletePrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Mark a private message as read.
pub struct MarkPrivateMessageAsRead {
  pub private_message_id: PrivateMessageId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Get your private messages.
pub struct GetPrivateMessages {
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub creator_id: Option<PersonId>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The private messages response.
pub struct PrivateMessagesResponse {
  pub private_messages: Vec<PrivateMessageView>,
}

#[lemmy_dto]
/// A single private message response.
pub struct PrivateMessageResponse {
  pub private_message_view: PrivateMessageView,
}

#[lemmy_dto(Default)]
/// Create a report for a private message.
pub struct CreatePrivateMessageReport {
  pub private_message_id: PrivateMessageId,
  pub reason: String,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// A private message report response.
pub struct PrivateMessageReportResponse {
  pub private_message_report_view: PrivateMessageReportView,
}

#[lemmy_dto(Default)]
/// Resolve a private message report.
pub struct ResolvePrivateMessageReport {
  pub report_id: PrivateMessageReportId,
  pub resolved: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// List private message reports.
// TODO , perhaps GetReports should be a tagged enum list too.
pub struct ListPrivateMessageReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The response for list private message reports.
pub struct ListPrivateMessageReportsResponse {
  pub private_message_reports: Vec<PrivateMessageReportView>,
}
