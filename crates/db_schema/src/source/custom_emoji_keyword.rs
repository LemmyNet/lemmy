#[cfg(feature = "full")]
use crate::schema::custom_emoji_keyword;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji_keyword))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::custom_emoji::CustomEmoji))
)]
pub struct CustomEmojiKeyword {
  pub id: i32,
  pub custom_emoji_id: i32,
  pub keyword: String,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji_keyword))]
pub struct CustomEmojiKeywordInsertForm {
  pub custom_emoji_id: i32,
  pub keyword: String,
}
