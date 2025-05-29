use crate::{
  newtypes::{DbUrl, InstanceId, MultiCommunityId, PersonId},
  sensitive::SensitiveString,
};
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
  pub instance_id: InstanceId,
  pub name: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub title: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  pub local: bool,
  pub deleted: bool,
  pub ap_id: DbUrl,
  pub public_key: String,
  pub private_key: Option<SensitiveString>,
  pub inbox_url: DbUrl,
  pub last_refreshed_at: DateTime<Utc>,
  pub following_url: DbUrl,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
pub struct MultiCommunityInsertForm {
  pub creator_id: PersonId,
  pub instance_id: InstanceId,
  pub name: String,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub title: Option<String>,
  #[new(default)]
  pub description: Option<String>,
  #[new(default)]
  pub public_key: Option<String>,
  #[new(default)]
  pub last_refreshed_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub private_key: Option<SensitiveString>,
  #[new(default)]
  pub inbox_url: Option<DbUrl>,
  #[new(default)]
  pub following_url: Option<DbUrl>,
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MultiCommunityApub {
  pub multi: MultiCommunity,
  pub entries: Vec<DbUrl>,
}
