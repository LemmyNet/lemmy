use crate::schema::local_user;
use serde::Serialize;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "local_user"]
pub struct LocalUser {
  pub id: i32,
  pub person_id: i32,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub admin: bool,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
}

// TODO redo these, check table defaults
#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "local_user"]
pub struct LocalUserForm {
  pub person_id: i32,
  pub password_encrypted: String,
  pub email: Option<Option<String>>,
  pub admin: Option<bool>,
  pub show_nsfw: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<i16>,
  pub default_listing_type: Option<i16>,
  pub lang: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub matrix_user_id: Option<Option<String>>,
}

/// A local user view that removes password encrypted
#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "local_user"]
pub struct LocalUserSettings {
  pub id: i32,
  pub person_id: i32,
  pub email: Option<String>,
  pub admin: bool,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
}
