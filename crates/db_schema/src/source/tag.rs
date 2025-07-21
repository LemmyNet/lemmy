use crate::newtypes::{CommunityId, DbUrl, PostId, TagId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{sql_types::Nullable, AsExpression, FromSqlRow};
use lemmy_db_schema_file::schema::post_tag;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::tag;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A tag that can be assigned to a post within a community.
/// The tag object is created by the community moderators.
/// The assignment happens by the post creator and can be updated by the community moderators.
///
/// A tag is a federated object that gives additional context to another object, which can be
/// displayed and filtered on. Currently, we only have community post tags, which is a tag that is
/// created by the mods of a community, then assigned to posts by post authors as well as mods of a
/// community, to categorize a post.
///
/// In the future we may add more tag types, depending on the requirements, this will lead to either
/// expansion of this table (community_id optional, addition of tag_type enum) or split of this
/// table / creation of new tables.
pub struct Tag {
  pub id: TagId,
  pub ap_id: DbUrl,
  pub display_name: String,
  /// the community that owns this tag
  pub community_id: CommunityId,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub deleted: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
pub struct TagInsertForm {
  pub ap_id: DbUrl,
  pub display_name: String,
  pub community_id: CommunityId,
  pub deleted: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
pub struct TagUpdateForm {
  pub ap_id: Option<DbUrl>,
  pub display_name: Option<String>,
  pub community_id: Option<CommunityId>,
  pub published_at: Option<DateTime<Utc>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "full", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<diesel::sql_types::Json>))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// we wrap this in a struct so we can implement FromSqlRow<Json> for it
pub struct TagsView(pub Vec<Tag>);

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
