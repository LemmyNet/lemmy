use crate::newtypes::{DbUrl, LocalUserId};
#[cfg(feature = "full")]
use crate::schema::{image_details, local_image, remote_image};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, Associations, TS)
)]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(table_name = local_image))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_user::LocalUser))
)]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(primary_key(pictrs_alias)))]
pub struct LocalImage {
  pub local_user_id: Option<LocalUserId>,
  pub pictrs_alias: String,
  pub pictrs_delete_token: String,
  pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_image))]
pub struct LocalImageForm {
  pub local_user_id: Option<LocalUserId>,
  pub pictrs_alias: String,
  pub pictrs_delete_token: String,
}

/// Stores all images which are hosted on remote domains. When attempting to proxy an image, it
/// is checked against this table to avoid Lemmy being used as a general purpose proxy.
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = remote_image))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(primary_key(link)))]
pub struct RemoteImage {
  pub link: DbUrl,
  pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = remote_image))]
pub struct RemoteImageForm {
  pub link: DbUrl,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(table_name = image_details))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(primary_key(link)))]
pub struct ImageDetails {
  pub link: DbUrl,
  pub width: i32,
  pub height: i32,
  pub content_type: String,
  pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = image_details))]
pub struct ImageDetailsForm {
  pub link: DbUrl,
  pub width: i32,
  pub height: i32,
  pub content_type: String,
}
