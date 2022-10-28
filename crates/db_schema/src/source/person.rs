use crate::newtypes::{DbUrl, InstanceId, PersonId};
#[cfg(feature = "full")]
use crate::schema::person;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
pub struct Person {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: String,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
  pub ban_expires: Option<chrono::NaiveDateTime>,
  pub instance_id: InstanceId,
}

/// A safe representation of person, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
pub struct PersonSafe {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
  pub ban_expires: Option<chrono::NaiveDateTime>,
  pub instance_id: InstanceId,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
pub struct PersonInsertForm {
  #[builder(!default)]
  pub name: String,
  #[builder(!default)]
  pub public_key: String,
  #[builder(!default)]
  pub instance_id: InstanceId,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: Option<DbUrl>,
  pub bio: Option<String>,
  pub local: Option<bool>,
  pub private_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub banner: Option<DbUrl>,
  pub deleted: Option<bool>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: Option<bool>,
  pub bot_account: Option<bool>,
  pub ban_expires: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
#[builder(field_defaults(default))]
pub struct PersonUpdateForm {
  pub display_name: Option<Option<String>>,
  pub avatar: Option<Option<DbUrl>>,
  pub banned: Option<bool>,
  pub updated: Option<Option<chrono::NaiveDateTime>>,
  pub actor_id: Option<DbUrl>,
  pub bio: Option<Option<String>>,
  pub local: Option<bool>,
  pub public_key: Option<String>,
  pub private_key: Option<Option<String>>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub banner: Option<Option<DbUrl>>,
  pub deleted: Option<bool>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<Option<DbUrl>>,
  pub matrix_user_id: Option<Option<String>>,
  pub admin: Option<bool>,
  pub bot_account: Option<bool>,
  pub ban_expires: Option<Option<chrono::NaiveDateTime>>,
}
