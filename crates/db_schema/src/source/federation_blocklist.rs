use crate::newtypes::{InstanceId, PersonId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(feature = "full")]
use {
  crate::schema::{admin_block_instance, federation_blocklist},
  ts_rs::TS,
};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(TS, Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct FederationBlockList {
  pub instance_id: InstanceId,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub block_expires: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
pub(crate) struct FederationBlockListForm {
  pub instance_id: InstanceId,
  pub updated: Option<DateTime<Utc>>,
  pub block_expires: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(TS, Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = admin_block_instance))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminBlockInstance {
  pub id: i32,
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<DateTime<Utc>>,
  pub published: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_block_instance))]
pub struct AdminBlockInstanceForm {
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub reason: Option<String>,
  pub expires: Option<DateTime<Utc>>,
}
