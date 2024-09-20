use crate::newtypes::{CustomEmojiId, DbUrl};
#[cfg(feature = "full")]
use crate::schema::custom_emoji;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A custom emoji.
pub struct CustomEmoji {
  pub id: CustomEmojiId,
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
pub struct CustomEmojiInsertForm {
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
pub struct CustomEmojiUpdateForm {
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
}
