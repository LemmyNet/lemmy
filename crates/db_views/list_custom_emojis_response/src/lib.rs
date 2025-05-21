use lemmy_db_views_custom_emoji::CustomEmojiView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for custom emojis.
pub struct ListCustomEmojisResponse {
  pub custom_emojis: Vec<CustomEmojiView>,
}
