use crate::newtypes::{PersonId, PrivateMessageId, PrivateMessageReportId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::private_message_report;

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::private_message::PrivateMessage))
)]
#[cfg_attr(feature = "full", diesel(table_name = private_message_report))]
pub struct PrivateMessageReport {
  pub id: PrivateMessageReportId,
  pub creator_id: PersonId,
  pub private_message_id: PrivateMessageId,
  pub original_pm_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
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
