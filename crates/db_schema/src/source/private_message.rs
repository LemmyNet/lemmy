use crate::newtypes::{DbUrl, PersonId, PrivateMessageId};
#[cfg(feature = "full")]
use crate::schema::private_message;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::person::Person, foreign_key = creator_id)
))] // Is this the right assoc?
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message.
pub struct PrivateMessage {
  pub id: PrivateMessageId,
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  pub deleted: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
pub struct PrivateMessageInsertForm {
  #[builder(!default)]
  pub creator_id: PersonId,
  #[builder(!default)]
  pub recipient_id: PersonId,
  #[builder(!default)]
  pub content: String,
  pub deleted: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = private_message))]
pub struct PrivateMessageUpdateForm {
  pub content: Option<String>,
  pub deleted: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<Option<chrono::NaiveDateTime>>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}
