use crate::newtypes::{InstanceId, PersonId};
#[cfg(feature = "full")]
use crate::schema::instance_block;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = instance_block))]
pub struct InstanceBlock {
  pub id: i32,
  pub person_id: PersonId,
  pub instance_id: InstanceId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = instance_block))]
pub struct InstanceBlockForm {
  pub person_id: PersonId,
  pub instance_id: InstanceId,
}
