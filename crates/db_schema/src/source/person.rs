#[cfg(feature = "full")]
use crate::schema::{person, person_follower};
use crate::{
  newtypes::{DbUrl, InstanceId, PersonId},
  sensitive::SensitiveString,
  source::placeholder_apub_url,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person.
pub struct Person {
  pub id: PersonId,
  pub name: String,
  /// A shorter display name.
  pub display_name: Option<String>,
  /// A URL for an avatar.
  pub avatar: Option<DbUrl>,
  /// Whether the person is banned.
  pub banned: bool,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  /// The federated actor_id.
  pub actor_id: DbUrl,
  /// An optional bio, in markdown.
  pub bio: Option<String>,
  /// Whether the person is local to our site.
  pub local: bool,
  #[serde(skip)]
  pub private_key: Option<SensitiveString>,
  #[serde(skip)]
  pub public_key: String,
  #[serde(skip)]
  pub last_refreshed_at: DateTime<Utc>,
  /// A URL for a banner.
  pub banner: Option<DbUrl>,
  /// Whether the person is deleted.
  pub deleted: bool,
  #[cfg_attr(feature = "full", ts(skip))]
  #[serde(skip, default = "placeholder_apub_url")]
  pub inbox_url: DbUrl,
  #[serde(skip)]
  pub shared_inbox_url: Option<DbUrl>,
  /// A matrix id, usually given an @person:matrix.org
  pub matrix_user_id: Option<String>,
  /// Whether the person is a bot account.
  pub bot_account: bool,
  /// When their ban, if it exists, expires, if at all.
  pub ban_expires: Option<DateTime<Utc>>,
  pub instance_id: InstanceId,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
pub struct PersonInsertForm {
  pub name: String,
  pub public_key: String,
  pub instance_id: InstanceId,
  #[new(default)]
  pub display_name: Option<String>,
  #[new(default)]
  pub avatar: Option<DbUrl>,
  #[new(default)]
  pub banned: Option<bool>,
  #[new(default)]
  pub published: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated: Option<DateTime<Utc>>,
  #[new(default)]
  pub actor_id: Option<DbUrl>,
  #[new(default)]
  pub bio: Option<String>,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub private_key: Option<String>,
  #[new(default)]
  pub last_refreshed_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub banner: Option<DbUrl>,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub inbox_url: Option<DbUrl>,
  #[new(default)]
  pub shared_inbox_url: Option<DbUrl>,
  #[new(default)]
  pub matrix_user_id: Option<String>,
  #[new(default)]
  pub bot_account: Option<bool>,
  #[new(default)]
  pub ban_expires: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
pub struct PersonUpdateForm {
  pub display_name: Option<Option<String>>,
  pub avatar: Option<Option<DbUrl>>,
  pub banned: Option<bool>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub actor_id: Option<DbUrl>,
  pub bio: Option<Option<String>>,
  pub local: Option<bool>,
  pub public_key: Option<String>,
  pub private_key: Option<Option<String>>,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub banner: Option<Option<DbUrl>>,
  pub deleted: Option<bool>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<Option<DbUrl>>,
  pub matrix_user_id: Option<Option<String>>,
  pub bot_account: Option<bool>,
  pub ban_expires: Option<Option<DateTime<Utc>>>,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(table_name = person_follower))]
#[cfg_attr(feature = "full", diesel(primary_key(follower_id, person_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PersonFollower {
  pub person_id: PersonId,
  pub follower_id: PersonId,
  pub published: DateTime<Utc>,
  pub pending: bool,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_follower))]
pub struct PersonFollowerForm {
  pub person_id: PersonId,
  pub follower_id: PersonId,
  pub pending: bool,
}
