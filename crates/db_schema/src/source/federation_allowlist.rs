use crate::newtypes::{InstanceId, PersonId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(feature = "full")]
use {
  crate::schema::{admin_allow_instance, federation_allowlist},
  ts_rs::TS,
};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = federation_allowlist))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct FederationAllowList {
  pub instance_id: InstanceId,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_allowlist))]
pub(crate) struct FederationAllowListForm {
  pub instance_id: InstanceId,
  pub updated: Option<DateTime<Utc>>,
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
#[cfg_attr(feature = "full", diesel(table_name = admin_allow_instance))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminAllowInstance {
  pub id: i32,
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub published: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_allow_instance))]
pub struct AdminAllowInstanceForm {
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub reason: Option<String>,
}
