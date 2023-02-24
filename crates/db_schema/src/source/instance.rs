use crate::newtypes::InstanceId;
#[cfg(feature = "full")]
use crate::schema::instance;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typed_builder::TypedBuilder;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
pub struct Instance {
  pub id: InstanceId,
  pub domain: String,
  pub software: Option<String>,
  pub version: Option<String>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
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
  pub updated: Option<chrono::NaiveDateTime>,
}
