use crate::sensitive::Sensitive;
use lemmy_db_schema::newtypes::{PersonId, PrivateMessageId, PrivateMessageReportId};
use lemmy_db_views::structs::{PrivateMessageReportView, PrivateMessageView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessage {
  pub content: String,
  pub recipient_id: PersonId,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditPrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub content: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeletePrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkPrivateMessageAsRead {
  pub private_message_id: PrivateMessageId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPrivateMessages {
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessagesResponse {
  pub private_messages: Vec<PrivateMessageView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageResponse {
  pub private_message_view: PrivateMessageView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessageReport {
  pub private_message_id: PrivateMessageId,
  pub reason: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageReportResponse {
  pub private_message_report_view: PrivateMessageReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResolvePrivateMessageReport {
  pub report_id: PrivateMessageReportId,
  pub resolved: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListPrivateMessageReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListPrivateMessageReportsResponse {
  pub private_message_reports: Vec<PrivateMessageReportView>,
}
