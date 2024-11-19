use crate::newtypes::{InstanceId, PersonId};
#[cfg(feature = "full")]
use crate::schema::federation_blocklist;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use ts_rs::TS;

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
  pub updated: Option<DateTime<Utc>>,
  // TODO: would be good to make this mandatory but value doesnt exist for old entries
  pub admin_person_id: Option<PersonId>,
  pub reason: Option<String>,
  pub expires: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
pub struct FederationBlockListForm {
  pub instance_id: InstanceId,
  pub updated: Option<DateTime<Utc>>,
  pub admin_person_id: Option<PersonId>,
  pub reason: Option<String>,
  pub expires: Option<DateTime<Utc>>,
}
