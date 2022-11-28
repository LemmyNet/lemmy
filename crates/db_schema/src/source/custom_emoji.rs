use crate::newtypes::{DbUrl, LocalSiteId};
#[cfg(feature = "full")]
use crate::schema::custom_emoji;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_site::LocalSite))
)]
pub struct CustomEmoji {
  pub id: i32,
  pub local_site_id: LocalSiteId,
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
pub struct CustomEmojiInsertForm {
  pub local_site_id: LocalSiteId,
  pub shortcode: String,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = custom_emoji))]
pub struct CustomEmojiUpdateForm {
  pub local_site_id: LocalSiteId,
  pub image_url: DbUrl,
  pub alt_text: String,
  pub category: String,
}
