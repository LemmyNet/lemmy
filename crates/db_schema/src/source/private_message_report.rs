use crate::newtypes::{PersonId, PrivateMessageId, PrivateMessageReportId};
#[cfg(feature = "full")]
use crate::schema::private_message_report;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::private_message::PrivateMessage))
)]
#[cfg_attr(feature = "full", diesel(table_name = private_message_report))]
#[cfg_attr(feature = "full", ts(export))]
/// The private message report.
pub struct PrivateMessageReport {
  pub id: PrivateMessageReportId,
  pub creator_id: PersonId,
  pub private_message_id: PrivateMessageId,
  /// The original text.
  pub original_pm_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = private_message_report))]
pub struct PrivateMessageReportForm {
  pub creator_id: PersonId,
  pub private_message_id: PrivateMessageId,
  pub original_pm_text: String,
  pub reason: String,
}
