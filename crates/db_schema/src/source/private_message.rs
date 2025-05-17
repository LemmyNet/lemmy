use crate::newtypes::{DbUrl, PersonId, PrivateMessageId};
#[cfg(feature = "full")]
use crate::schema::private_message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::person::Person, foreign_key = creator_id)
))] // Is this the right assoc?
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message.
pub struct PrivateMessage {
  pub id: PrivateMessageId,
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  pub deleted: bool,
  pub read: bool,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
pub struct PrivateMessageInsertForm {
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub read: Option<bool>,
  #[new(default)]
  pub published: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated: Option<DateTime<Utc>>,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
  #[new(default)]
  pub local: Option<bool>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
pub struct PrivateMessageUpdateForm {
  pub content: Option<String>,
  pub deleted: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}
