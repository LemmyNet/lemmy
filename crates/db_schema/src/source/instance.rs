use crate::newtypes::InstanceId;
#[cfg(feature = "full")]
use crate::schema::instance;
use std::fmt::Debug;
use typed_builder::TypedBuilder;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
pub struct Instance {
  pub id: InstanceId,
  pub domain: String,
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
  pub updated: Option<chrono::NaiveDateTime>,
}
