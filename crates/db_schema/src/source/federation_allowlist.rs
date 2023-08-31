use crate::newtypes::InstanceId;
#[cfg(feature = "full")]
use crate::schema::federation_allowlist;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = federation_allowlist))]
pub struct FederationAllowList {
  pub id: i32,
  pub instance_id: InstanceId,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_allowlist))]
pub struct FederationAllowListForm {
  pub instance_id: InstanceId,
  pub updated: Option<DateTime<Utc>>,
}
