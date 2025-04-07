use crate::newtypes::{CommunityId, MultiCommunityId, PersonId};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{multi_community, multi_community_entry};
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
  pub owner_id: PersonId,
  pub name: String,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community))]
pub struct MultiCommunityInsertForm {
  pub id: MultiCommunityId,
  pub owner_id: PersonId,
  pub name: String,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = multi_community_entry))]
pub struct MultiCommunityEntryForm {
  pub multi_community_id: MultiCommunityId,
  pub community_id: CommunityId,
}
