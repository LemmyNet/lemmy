use lemmy_db_schema::source::{
  custom_emoji::CustomEmoji,
  custom_emoji_keyword::CustomEmojiKeyword,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {diesel::Queryable, ts_rs::TS};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A custom emoji view.
pub struct CustomEmojiView {
  pub custom_emoji: CustomEmoji,
  pub keywords: Vec<CustomEmojiKeyword>,
}
