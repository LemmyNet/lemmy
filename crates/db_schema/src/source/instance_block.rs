use crate::newtypes::{InstanceId, PersonId};
#[cfg(feature = "full")]
use crate::schema::instance_actions;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{dsl, expression_methods::NullableExpressionMethods};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = instance_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct InstanceBlock {
  pub person_id: PersonId,
  pub instance_id: InstanceId,
  #[diesel(select_expression = instance_actions::blocked.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<instance_actions::blocked>)]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = instance_actions))]
pub struct InstanceBlockForm {
  pub person_id: PersonId,
  pub instance_id: InstanceId,
}
