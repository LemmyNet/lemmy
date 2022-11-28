use crate::sensitive::Sensitive;
use lemmy_db_views::structs::CustomEmojiView;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateCustomEmoji {
  pub category: String,
  pub shortcode: String,
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditCustomEmoji {
  pub id: i32,
  pub category: String,
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeleteCustomEmoji {
  pub id: i32,
  pub auth: Sensitive<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteCustomEmojiResponse {
  pub id: i32,
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomEmojiResponse {
  pub custom_emoji: CustomEmojiView,
}
