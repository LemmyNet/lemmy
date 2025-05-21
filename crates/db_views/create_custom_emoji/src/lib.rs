use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create a custom emoji.
pub struct CreateCustomEmoji {
  pub category: String,
  pub shortcode: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
}
