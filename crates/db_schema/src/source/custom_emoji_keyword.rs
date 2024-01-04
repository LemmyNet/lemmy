use crate::newtypes::CustomEmojiId;
#[cfg(feature = "full")]
use crate::schema::custom_emoji_keyword;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji_keyword))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::custom_emoji::CustomEmoji))
)]
#[cfg_attr(feature = "full", diesel(primary_key(custom_emoji_id, keyword)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A custom keyword for an emoji.
pub struct CustomEmojiKeyword {
  pub custom_emoji_id: CustomEmojiId,
  pub keyword: String,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji_keyword))]
pub struct CustomEmojiKeywordInsertForm {
  pub custom_emoji_id: CustomEmojiId,
  pub keyword: String,
}
