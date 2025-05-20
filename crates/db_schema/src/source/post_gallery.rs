use crate::newtypes::{DbUrl, PostGalleryId, PostId};
use chrono::{DateTime, Utc};
use diesel::sql_types;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {lemmy_db_schema_file::schema::post_gallery, ts_rs::TS};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, TS, Identifiable))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(table_name = post_gallery))]
#[cfg_attr(feature = "full", diesel(belongs_to(post)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PostGallery {
  pub id: PostGalleryId,
  #[serde(skip)]
  pub post_id: PostId,
  pub url: DbUrl,
  pub page: i32,
  // An optional alt_text, usable for image posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub alt_text: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub caption: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub url_content_type: Option<String>,
  #[serde(skip)]
  pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_gallery))]
pub struct PostGalleryInsertForm {
  pub post_id: PostId,
  pub page: i32,
  pub url: DbUrl,
  #[new(default)]
  pub url_content_type: Option<String>,
  #[new(default)]
  pub caption: Option<String>,
  #[new(default)]
  pub alt_text: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
#[cfg_attr(feature = "full", derive(TS, FromSqlRow, AsExpression))]
#[serde(transparent)]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<sql_types::Json>))]
pub struct PostGalleryView(pub Vec<PostGallery>);
