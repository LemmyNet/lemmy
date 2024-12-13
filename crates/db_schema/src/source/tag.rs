use crate::newtypes::{CommunityId, DbUrl, PostId, TagId};
#[cfg(feature = "full")]
use crate::schema::{post_tag, tag};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

/// A tag that can be assigned to a post within a community.
/// The tag object is created by the community moderators.
/// The assignment happens by the post creator and can be updated by the community moderators.
///
/// A tag is a federatable object that gives additional context to another object, which can be
/// displayed and filtered on currently, we only have community post tags, which is a tag that is
/// created by post authors as well as mods  of a community, to categorize a post. in the future we
/// may add more tag types, depending on the requirements, this will lead to either expansion of
/// this table (community_id optional, addition of tag_type enum) or split of this table / creation
/// of new tables.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct Tag {
  pub id: TagId,
  pub ap_id: DbUrl,
  pub name: String,
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
  pub name: String,
  pub community_id: CommunityId,
  // default now
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_tag))]
pub struct PostTagInsertForm {
  pub post_id: PostId,
  pub tag_id: TagId,
}
