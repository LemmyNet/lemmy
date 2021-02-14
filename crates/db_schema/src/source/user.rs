use crate::{
  schema::{user_, user_alias_1, user_alias_2},
  Url,
};
use serde::Serialize;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_"]
pub struct User_ {
  pub id: i32,                                  // person
  pub name: String,                             // person
  pub preferred_username: Option<String>,       // person
  pub password_encrypted: String,               // local_user
  pub email: Option<String>,                    // local_user
  pub avatar: Option<String>,                   // person
  pub admin: bool,                              // local_user
  pub banned: bool,                             // person?
  pub published: chrono::NaiveDateTime,         // person
  pub updated: Option<chrono::NaiveDateTime>,   // person
  pub show_nsfw: bool,                          // local_user
  pub theme: String,                            // local_user
  pub default_sort_type: i16,                   // local_user
  pub default_listing_type: i16,                // local_user
  pub lang: String,                             // local_user
  pub show_avatars: bool,                       // local_user
  pub send_notifications_to_email: bool,        // local_user
  pub matrix_user_id: Option<String>,           // local_user
  pub actor_id: Url,                            // person
  pub bio: Option<String>,                      // person
  pub local: bool,                              // person
  pub private_key: Option<String>,              // person
  pub public_key: Option<String>,               // person
  pub last_refreshed_at: chrono::NaiveDateTime, // person
  pub banner: Option<String>,                   // person
  pub deleted: bool,                            // person
  pub inbox_url: Url,                           // person
  pub shared_inbox_url: Option<Url>,            // person
}

/// A safe representation of user, without the sensitive info
#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_"]
pub struct UserSafe {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub matrix_user_id: Option<String>,
  pub actor_id: Url,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<String>,
  pub deleted: bool,
  pub inbox_url: Url,
  pub shared_inbox_url: Option<Url>,
}

/// A safe user view with only settings
#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_"]
pub struct UserSafeSettings {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub email: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: Url,
  pub bio: Option<String>,
  pub local: bool,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<String>,
  pub deleted: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_alias_1"]
pub struct UserAlias1 {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: Url,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<String>,
  pub deleted: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_alias_1"]
pub struct UserSafeAlias1 {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub matrix_user_id: Option<String>,
  pub actor_id: Url,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<String>,
  pub deleted: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_alias_2"]
pub struct UserAlias2 {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: Url,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<String>,
  pub deleted: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "user_alias_2"]
pub struct UserSafeAlias2 {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub matrix_user_id: Option<String>,
  pub actor_id: Url,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<String>,
  pub deleted: bool,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "user_"]
pub struct UserForm {
  pub name: String,
  pub preferred_username: Option<Option<String>>,
  pub password_encrypted: String,
  pub admin: bool,
  pub banned: Option<bool>,
  pub email: Option<Option<String>>,
  pub avatar: Option<Option<String>>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<Option<String>>,
  pub actor_id: Option<Url>,
  pub bio: Option<Option<String>>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub banner: Option<Option<String>>,
  pub inbox_url: Option<Url>,
  pub shared_inbox_url: Option<Option<Url>>,
}
