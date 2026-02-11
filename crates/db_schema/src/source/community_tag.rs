use crate::newtypes::{CommunityId, CommunityTagId, PostId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{AsExpression, FromSqlRow, sql_types::Nullable};
use lemmy_db_schema_file::enums::TagColor;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{community_tag, post_community_tag};
use lemmy_diesel_utils::dburl::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A tag that is created by community moderators, and assigned to posts by the creator
/// or by mods.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = community_tag))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct CommunityTag {
  pub id: CommunityTagId,
  pub ap_id: DbUrl,
  pub name: String,
  pub display_name: Option<String>,
  pub summary: Option<String>,
  /// The community that this tag belongs to
  pub community_id: CommunityId,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub deleted: bool,
  pub color: TagColor,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_tag))]
pub struct CommunityTagInsertForm {
  pub ap_id: DbUrl,
  pub name: String,
  pub display_name: Option<String>,
  pub summary: Option<String>,
  pub community_id: CommunityId,
  pub deleted: Option<bool>,
  pub color: Option<TagColor>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_tag))]
pub struct CommunityTagUpdateForm {
  pub display_name: Option<Option<String>>,
  pub summary: Option<Option<String>>,
  pub community_id: Option<CommunityId>,
  pub published_at: Option<DateTime<Utc>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub color: Option<TagColor>,
}

/// We wrap this in a struct so we can implement FromSqlRow<Json> for it
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "full", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "full", diesel(sql_type = Nullable<diesel::sql_types::Json>))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct CommunityTagsView(pub Vec<CommunityTag>);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community_tag::CommunityTag))
)]
#[cfg_attr(feature = "full", diesel(table_name = post_community_tag))]
#[cfg_attr(feature = "full", diesel(primary_key(post_id, community_tag_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// An association between a post and a tag. Created/updated by the post author or mods of a
/// community.
pub struct PostCommunityTag {
  pub post_id: PostId,
  pub community_tag_id: CommunityTagId,
  pub published_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_community_tag))]
pub struct PostCommunityTagForm {
  pub post_id: PostId,
  pub community_tag_id: CommunityTagId,
}
