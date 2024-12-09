use crate::newtypes::{CommunityId, DbUrl, PostId, TagId};
#[cfg(feature = "full")]
use crate::schema::{community_post_tag, post_tag, tag};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

/// A tag that can be assigned to a post within a community.
/// The tag object is created by the community moderators.
/// The assignment happens by the post creator and can be updated by the community moderators.
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
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = community_post_tag))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", diesel(primary_key(community_id, tag_id)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommunityPostTag {
  pub community_id: CommunityId,
  pub tag_id: TagId,
  pub published: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tag))]
pub struct TagInsertForm {
  pub ap_id: DbUrl,
  pub name: String,
  // default now
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_post_tag))]
pub struct CommunityPostTagInsertForm {
  pub community_id: CommunityId,
  pub tag_id: TagId,
  pub published: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_tag))]
pub struct PostTagInsertForm {
  pub post_id: PostId,
  pub tag_id: TagId,
}
