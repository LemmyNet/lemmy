use crate::{newtypes::InstanceId, schema::instance};
use std::fmt::Debug;

#[derive(PartialEq, Eq, Debug, Queryable, Identifiable)]
#[diesel(table_name = instance)]
pub struct Instance {
  pub id: InstanceId,
  pub domain: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = instance)]
pub struct InstanceForm {
  pub domain: String,
  pub updated: Option<chrono::NaiveDateTime>,
}
