use crate::newtypes::InstanceId;
#[cfg(feature = "full")]
use crate::schema::instance;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
#[cfg_attr(feature = "full", ts(export))]
/// A federated instance / site.
pub struct Instance {
  pub id: InstanceId,
  pub domain: String,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  pub software: Option<String>,
  pub version: Option<String>,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
pub struct InstanceForm {
  #[builder(!default)]
  pub domain: String,
  pub software: Option<String>,
  pub version: Option<String>,
  pub updated: Option<DateTime<Utc>>,
}
