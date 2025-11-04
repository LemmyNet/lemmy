use crate::newtypes::{PersonId, PostId};
use chrono::{DateTime, Utc};
use lemmy_diesel_utils::dburl::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
#[cfg(feature = "full")]
use {
  i_love_jesus::CursorKeysModule,
  lemmy_db_schema_file::schema::{image_details, local_image, remote_image},
};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, Associations, CursorKeysModule,)
)]
#[cfg_attr(feature = "full", diesel(table_name = local_image))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(primary_key(pictrs_alias)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = local_image_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct LocalImage {
  pub pictrs_alias: String,
  pub published_at: DateTime<Utc>,
  pub person_id: Option<PersonId>,
  /// This means the image is an auto-generated thumbnail, for a post.
  pub thumbnail_for_post_id: Option<PostId>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_image))]
pub struct LocalImageForm {
  pub pictrs_alias: String,
  pub person_id: PersonId,
  pub thumbnail_for_post_id: Option<Option<PostId>>,
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
  pub published_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = image_details))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(primary_key(link)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ImageDetails {
  pub link: DbUrl,
  pub width: i32,
  pub height: i32,
  pub content_type: String,
  pub blurhash: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = image_details))]
pub struct ImageDetailsInsertForm {
  pub link: DbUrl,
  pub width: i32,
  pub height: i32,
  pub content_type: String,
  pub blurhash: Option<String>,
}
