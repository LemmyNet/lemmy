use crate::newtypes::{CommunityId, DbUrl, TagId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{sql_types::Nullable, AsExpression, FromSqlRow},
  lemmy_db_schema_file::schema::tag,
  ts_rs::TS,
};

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A tag that can be assigned to a post within a community.
/// The tag object is created by the community moderators.
/// The assignment happens by the post creator and can be updated by the community moderators.
///
/// A tag is a federatable object that gives additional context to another object, which can be
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
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
  pub deleted: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
pub struct TagInsertForm {
  pub ap_id: DbUrl,
  pub display_name: String,
  pub community_id: CommunityId,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
pub struct TagUpdateForm {
  pub ap_id: Option<DbUrl>,
  pub display_name: Option<String>,
  pub community_id: Option<CommunityId>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "full", derive(TS, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<diesel::sql_types::Json>))]
/// we wrap this in a struct so we can implement FromSqlRow<Json> for it
pub struct TagsView(pub Vec<Tag>);
