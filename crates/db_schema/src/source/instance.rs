use crate::newtypes::InstanceId;
#[cfg(feature = "full")]
use crate::schema::instance;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
pub struct Instance {
  pub id: InstanceId,
  pub domain: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = instance))]
pub struct InstanceForm {
  pub domain: String,
  pub updated: Option<chrono::NaiveDateTime>,
}
