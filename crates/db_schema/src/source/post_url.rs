use crate::newtypes::{DbUrl, PostId, PostUrlId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use super::post::Post;

#[cfg(feature = "full")]
use {
  lemmy_db_schema_file::schema::post_url,
  ts_rs::TS,
};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, TS, Identifiable))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(table_name = post_url))]
#[cfg_attr(feature = "full", diesel(belongs_to(post)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PostUrl {
  pub id: PostUrlId,
  #[serde(skip)]
  pub post_id: PostId,
  pub url: DbUrl,
  pub page: i32,
  #[cfg_attr(feature = "full", ts(optional))]
  pub url_content_type: Option<String>,
  // An optional alt_text, usable for image posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub alt_text: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub caption: Option<String>,
  #[serde(skip)]
  pub published: DateTime<Utc>,
  #[serde(skip)]
  pub updated: Option<DateTime<Utc>>,
}

// #[skip_serializing_none]
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// #[cfg_attr(feature = "full", derive(TS))]
// #[cfg_attr(feature = "full", ts(export))]
// // #[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
// struct PostWithUrls {
//   #[serde(flatten)]
//   post: Post,
//   urls: Vec<PostUrl>
// }

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_url))]
pub struct PostUrlInsertForm {
  pub post_id: PostId,
  pub url: DbUrl,
  #[new(default)]
  pub url_content_type: Option<String>,
  #[new(default)]
  pub caption: Option<String>,
}
