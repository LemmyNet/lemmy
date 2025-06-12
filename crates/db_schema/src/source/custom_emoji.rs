use crate::newtypes::{CustomEmojiId, DbUrl};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::custom_emoji;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A custom emoji.
pub struct CustomEmoji {
  pub id: CustomEmojiId,
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
pub struct CustomEmojiInsertForm {
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
pub struct CustomEmojiUpdateForm {
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
}
