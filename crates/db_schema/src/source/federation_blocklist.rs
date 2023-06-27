use crate::newtypes::InstanceId;
#[cfg(feature = "full")]
use crate::schema::federation_blocklist;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
pub struct FederationBlockList {
  pub id: i32,
  pub instance_id: InstanceId,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
pub struct FederationBlockListForm {
  pub instance_id: InstanceId,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// A request to block an instance
pub struct BlockInstanceAction {
  pub domain: String,
  pub reason: Option<String>,
}
