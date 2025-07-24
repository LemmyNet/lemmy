use crate::newtypes::{DbUrl, PersonId, PrivateMessageId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::private_message;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::person::Person, foreign_key = creator_id)
))] // Is this the right assoc?
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A private message.
pub struct PrivateMessage {
  pub id: PrivateMessageId,
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  pub deleted: bool,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub ap_id: DbUrl,
  pub local: bool,
  pub removed: bool,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
pub struct PrivateMessageInsertForm {
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub published_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
  #[new(default)]
  pub local: Option<bool>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
pub struct PrivateMessageUpdateForm {
  pub content: Option<String>,
  pub deleted: Option<bool>,
  pub published_at: Option<DateTime<Utc>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub removed: Option<bool>,
}
