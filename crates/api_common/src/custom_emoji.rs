use lemmy_db_schema::newtypes::CustomEmojiId;
use lemmy_db_views::structs::CustomEmojiView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CreateCustomEmoji {
  pub category: String,
  pub shortcode: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct EditCustomEmoji {
  pub id: CustomEmojiId,
  pub category: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteCustomEmoji {
  pub id: CustomEmojiId,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteCustomEmojiResponse {
  pub id: CustomEmojiId,
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CustomEmojiResponse {
  pub custom_emoji: CustomEmojiView,
}
