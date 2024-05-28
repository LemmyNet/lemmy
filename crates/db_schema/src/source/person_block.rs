use crate::newtypes::PersonId;
#[cfg(feature = "full")]
use crate::schema::person_actions;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{dsl, expression_methods::NullableExpressionMethods};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(table_name = person_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, target_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PersonBlock {
  pub person_id: PersonId,
  pub target_id: PersonId,
  #[cfg_attr(feature = "full", diesel(select_expression = person_actions::blocked.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<person_actions::blocked>))]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_actions))]
pub struct PersonBlockForm {
  pub person_id: PersonId,
  pub target_id: PersonId,
}
