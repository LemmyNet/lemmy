use crate::newtypes::{InstanceId, PersonId};
#[cfg(feature = "full")]
use crate::schema::{instance, instance_actions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A federated instance / site.
pub struct Instance {
  pub id: InstanceId,
  pub domain: String,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When the instance was updated.
  pub updated: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The software of the instance.
  pub software: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The version of the instance's software.
  pub version: Option<String>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
pub struct InstanceForm {
  pub domain: String,
  #[new(default)]
  pub software: Option<String>,
  #[new(default)]
  pub version: Option<String>,
  #[new(default)]
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = instance_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InstanceActions {
  pub person_id: PersonId,
  pub instance_id: InstanceId,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When the instance was blocked.
  pub blocked: Option<DateTime<Utc>>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = instance_actions))]
pub struct InstanceBlockForm {
  pub person_id: PersonId,
  pub instance_id: InstanceId,
  #[new(value = "Utc::now()")]
  pub blocked: DateTime<Utc>,
}
