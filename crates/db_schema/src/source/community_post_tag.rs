use crate::{
  newtypes::{CommunityId, CommunityPostTagId, DbUrl, PostId},
  schema::{community_post_tag, post_community_post_tag},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ts_rs::TS;

/// A tag that can be assigned to a post within a community.
/// The tag object is created by the community moderators.
/// The assignment happens by the post creator and can be updated by the community moderators.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = community_post_tag))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommunityPostTag {
  pub id: CommunityPostTagId,
  pub ap_id: DbUrl,
  pub community_id: CommunityId,
  pub name: String,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_post_tag))]
pub struct CommunityPostTagInsertForm {
  pub ap_id: DbUrl,
  pub community_id: CommunityId,
  pub name: String,
  // default now
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_community_post_tag))]
pub struct PostCommunityPostTagInsertForm {
  pub post_id: PostId,
  pub community_post_tag_id: CommunityPostTagId,
}
