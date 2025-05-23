use crate::newtypes::{DbUrl, MultiCommunityId, PersonId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::multi_community;
use lemmy_db_schema_file::{enums::CommunityFollowerState, schema::multi_community_follow};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct MultiCommunity {
  pub id: MultiCommunityId,
  pub creator_id: PersonId,
  pub name: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub title: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  pub local: bool,
  pub deleted: bool,
  pub ap_id: DbUrl,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
pub struct MultiCommunityInsertForm {
  pub creator_id: PersonId,
  pub name: String,
  pub ap_id: DbUrl,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub title: Option<String>,
  #[new(default)]
  pub description: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
pub struct MultiCommunityUpdateForm {
  pub title: Option<Option<String>>,
  pub description: Option<Option<String>>,
  pub deleted: Option<bool>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community_follow))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct MultiCommunityFollow {
  pub multi_community_id: MultiCommunityId,
  pub person_id: PersonId,
  pub follow_state: CommunityFollowerState,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community_follow))]
pub struct MultiCommunityFollowForm {
  pub multi_community_id: MultiCommunityId,
  pub person_id: PersonId,
  pub follow_state: CommunityFollowerState,
}
