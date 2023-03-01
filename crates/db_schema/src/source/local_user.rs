use crate::newtypes::{LocalUserId, PersonId};
#[cfg(feature = "full")]
use crate::schema::local_user;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
pub struct LocalUser {
  pub id: LocalUserId,
  pub person_id: PersonId,
  #[serde(skip)]
  pub password_encrypted: String,
  pub email: Option<String>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub interface_language: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub validator_time: chrono::NaiveDateTime,
  pub show_bot_accounts: bool,
  pub show_scores: bool,
  pub show_read_posts: bool,
  pub show_new_post_notifs: bool,
  pub email_verified: bool,
  pub accepted_application: bool,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
pub struct LocalUserInsertForm {
  #[builder(!default)]
  pub person_id: PersonId,
  #[builder(!default)]
  pub password_encrypted: String,
  pub email: Option<String>,
  pub show_nsfw: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<i16>,
  pub default_listing_type: Option<i16>,
  pub interface_language: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub show_scores: Option<bool>,
  pub show_read_posts: Option<bool>,
  pub show_new_post_notifs: Option<bool>,
  pub email_verified: Option<bool>,
  pub accepted_application: Option<bool>,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
pub struct LocalUserUpdateForm {
  pub password_encrypted: Option<String>,
  pub email: Option<Option<String>>,
  pub show_nsfw: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<i16>,
  pub default_listing_type: Option<i16>,
  pub interface_language: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub show_scores: Option<bool>,
  pub show_read_posts: Option<bool>,
  pub show_new_post_notifs: Option<bool>,
  pub email_verified: Option<bool>,
  pub accepted_application: Option<bool>,
}
