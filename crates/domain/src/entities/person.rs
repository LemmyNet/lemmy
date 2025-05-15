use chrono::{DateTime, Utc};
use diesel::prelude::*;
use i_love_jesus::CursorKeysModule;
use lemmy_db_schema_file::schema::person;

use super::{
  newtypes::{DbUrl, InstanceId, PersonId},
  sensitive::SensitiveString,
};

#[derive(Clone, PartialEq, Eq, Debug, Queryable, Selectable, Identifiable, CursorKeysModule)]
#[diesel(table_name = person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cursor_keys_module(name = person_keys)]
/// A person.
pub struct Person {
  pub id: PersonId,
  pub name: String,
  /// A shorter display name.
  pub display_name: Option<String>,
  /// A URL for an avatar.
  pub avatar: Option<DbUrl>,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  /// The federated ap_id.
  pub ap_id: DbUrl,
  /// An optional bio, in markdown.
  pub bio: Option<String>,
  /// Whether the person is local to our site.
  pub local: bool,
  pub private_key: Option<SensitiveString>,
  pub public_key: String,
  pub last_refreshed_at: DateTime<Utc>,
  /// A URL for a banner.
  pub banner: Option<DbUrl>,
  /// Whether the person is deleted.
  pub deleted: bool,
  pub inbox_url: DbUrl,
  /// A matrix id, usually given an @person:matrix.org
  pub matrix_user_id: Option<String>,
  /// Whether the person is a bot account.
  pub bot_account: bool,
  pub instance_id: InstanceId,
  pub post_count: i64,
  pub post_score: i64,
  pub comment_count: i64,
  pub comment_score: i64,
}
