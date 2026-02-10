use crate::source::placeholder_apub_url;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{person, person_actions};
use lemmy_db_schema_file::{InstanceId, PersonId};
use lemmy_diesel_utils::{dburl::DbUrl, sensitive::SensitiveString};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = person))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person.
pub struct Person {
  pub id: PersonId,
  pub name: String,
  /// A shorter display name.
  pub display_name: Option<String>,
  /// A URL for an avatar.
  pub avatar: Option<DbUrl>,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// The federated ap_id.
  pub ap_id: DbUrl,
  /// An optional bio, in markdown.
  pub bio: Option<String>,
  /// Whether the person is local to our site.
  pub local: bool,
  #[serde(skip)]
  pub private_key: Option<SensitiveString>,
  #[serde(skip)]
  pub public_key: String,
  pub last_refreshed_at: DateTime<Utc>,
  /// A URL for a banner.
  pub banner: Option<DbUrl>,
  /// Whether the person is deleted.
  pub deleted: bool,
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  #[serde(skip, default = "placeholder_apub_url")]
  pub inbox_url: DbUrl,
  /// A matrix id, usually given an @person:matrix.org
  pub matrix_user_id: Option<String>,
  /// Whether the person is a bot account.
  pub bot_account: bool,
  pub instance_id: InstanceId,
  pub post_count: i32,
  #[serde(skip)]
  pub post_score: i32,
  pub comment_count: i32,
  #[serde(skip)]
  pub comment_score: i32,
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
  pub published_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
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
  pub matrix_user_id: Option<String>,
  #[new(default)]
  pub bot_account: Option<bool>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person))]
pub struct PersonUpdateForm {
  pub display_name: Option<Option<String>>,
  pub avatar: Option<Option<DbUrl>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub ap_id: Option<DbUrl>,
  pub bio: Option<Option<String>>,
  pub local: Option<bool>,
  pub public_key: Option<String>,
  pub private_key: Option<Option<String>>,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub banner: Option<Option<DbUrl>>,
  pub deleted: Option<bool>,
  pub inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<Option<String>>,
  pub bot_account: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(table_name = person_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, target_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PersonActions {
  #[serde(skip)]
  pub followed_at: Option<DateTime<Utc>>,
  /// When the person was blocked.
  pub blocked_at: Option<DateTime<Utc>>,
  #[serde(skip)]
  pub person_id: PersonId,
  #[serde(skip)]
  pub target_id: PersonId,
  #[serde(skip)]
  pub follow_pending: Option<bool>,
  /// When the person was noted.
  pub noted_at: Option<DateTime<Utc>>,
  /// A note about the person.
  pub note: Option<String>,
  /// When the person was voted on.
  pub voted_at: Option<DateTime<Utc>>,
  /// A total of upvotes given to this person
  pub upvotes: Option<i32>,
  /// A total of downvotes given to this person
  pub downvotes: Option<i32>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_actions))]
pub struct PersonFollowerForm {
  pub target_id: PersonId,
  pub person_id: PersonId,
  pub follow_pending: bool,
  #[new(value = "Utc::now()")]
  pub followed_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_actions))]
pub struct PersonBlockForm {
  pub person_id: PersonId,
  pub target_id: PersonId,
  #[new(value = "Utc::now()")]
  pub blocked_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_actions))]
pub struct PersonNoteForm {
  pub person_id: PersonId,
  pub target_id: PersonId,
  pub note: String,
  #[new(value = "Utc::now()")]
  pub noted_at: DateTime<Utc>,
}
