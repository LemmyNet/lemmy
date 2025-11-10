use crate::CustomEmojiView;
use lemmy_db_schema::newtypes::CustomEmojiId;
use lemmy_diesel_utils::dburl::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a custom emoji.
pub struct CreateCustomEmoji {
  pub category: String,
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for a custom emoji.
pub struct CustomEmojiResponse {
  pub custom_emoji: CustomEmojiView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete a custom emoji.
pub struct DeleteCustomEmoji {
  pub id: CustomEmojiId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edit a custom emoji.
pub struct EditCustomEmoji {
  pub id: CustomEmojiId,
  pub category: Option<String>,
  pub shortcode: Option<String>,
  pub image_url: Option<DbUrl>,
  pub alt_text: Option<String>,
  pub keywords: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches a list of custom emojis.
pub struct ListCustomEmojis {
  pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for custom emojis.
pub struct ListCustomEmojisResponse {
  pub custom_emojis: Vec<CustomEmojiView>,
}
