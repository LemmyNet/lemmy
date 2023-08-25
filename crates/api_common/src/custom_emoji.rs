use crate::sensitive::Sensitive;
use lemmy_db_schema::newtypes::CustomEmojiId;
use lemmy_db_views::structs::CustomEmojiView;
use lemmy_proc_macros::lemmy_dto;
use url::Url;

#[lemmy_dto]
/// Create a custom emoji.
pub struct CreateCustomEmoji {
  pub category: String,
  pub shortcode: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// Edit  a custom emoji.
pub struct EditCustomEmoji {
  pub id: CustomEmojiId,
  pub category: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub image_url: Url,
  pub alt_text: String,
  pub keywords: Vec<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Delete a custom emoji.
pub struct DeleteCustomEmoji {
  pub id: CustomEmojiId,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The response for deleting a custom emoji.
pub struct DeleteCustomEmojiResponse {
  pub id: CustomEmojiId,
  pub success: bool,
}

#[lemmy_dto]
/// A response for a custom emoji.
pub struct CustomEmojiResponse {
  pub custom_emoji: CustomEmojiView,
}
