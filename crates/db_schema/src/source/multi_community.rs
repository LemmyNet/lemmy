use crate::{
  newtypes::{CommunityId, MultiCommunityId},
  source::placeholder_apub_url,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{
  multi_community,
  multi_community_entry,
  multi_community_follow,
};
use lemmy_db_schema_file::{InstanceId, PersonId, enums::CommunityFollowerState};
use lemmy_diesel_utils::{dburl::DbUrl, sensitive::SensitiveString};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = multi_community_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct MultiCommunity {
  pub id: MultiCommunityId,
  pub creator_id: PersonId,
  pub instance_id: InstanceId,
  pub name: String,
  pub title: Option<String>,
  pub summary: Option<String>,
  pub local: bool,
  pub deleted: bool,
  pub ap_id: DbUrl,
  #[serde(skip)]
  pub public_key: String,
  #[serde(skip)]
  pub private_key: Option<SensitiveString>,
  #[serde(skip, default = "placeholder_apub_url")]
  pub inbox_url: DbUrl,
  #[serde(skip)]
  pub last_refreshed_at: DateTime<Utc>,
  #[serde(skip, default = "placeholder_apub_url")]
  pub following_url: DbUrl,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub subscribers: i32,
  pub subscribers_local: i32,
  pub communities: i32,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
pub struct MultiCommunityInsertForm {
  pub creator_id: PersonId,
  pub instance_id: InstanceId,
  pub name: String,
  pub public_key: String,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub title: Option<String>,
  #[new(default)]
  pub summary: Option<String>,
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
  pub summary: Option<Option<String>>,
  pub deleted: Option<bool>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community_follow))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
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

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community_entry))]
#[cfg_attr(
  feature = "full",
  diesel(primary_key(multi_community_id, community_id))
)]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct MultiCommunityEntry {
  pub multi_community_id: MultiCommunityId,
  pub community_id: CommunityId,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community_entry))]
pub struct MultiCommunityEntryForm {
  pub multi_community_id: MultiCommunityId,
  pub community_id: CommunityId,
}
