use crate::newtypes::{PostId, TagId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::post_tag;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::tag::Tag)))]
#[cfg_attr(feature = "full", diesel(table_name = post_tag))]
#[cfg_attr(feature = "full", diesel(primary_key(post_id, tag_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// An association between a post and a tag. Created/updated by the post author or mods of a
/// community. In the future, more access controls could be added, for example that specific tag
/// types can only be added by mods.
pub struct PostTag {
  pub post_id: PostId,
  pub tag_id: TagId,
  pub published_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_tag))]
pub struct PostTagForm {
  pub post_id: PostId,
  pub tag_id: TagId,
}
