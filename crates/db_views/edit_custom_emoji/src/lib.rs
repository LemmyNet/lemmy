use lemmy_db_schema::newtypes::CustomEmojiId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit  a custom emoji.
pub struct EditCustomEmoji {
  pub id: CustomEmojiId,
  pub category: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
}
