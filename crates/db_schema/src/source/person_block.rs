use crate::newtypes::{PersonBlockId, PersonId};
#[cfg(feature = "full")]
use crate::schema::person_block;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(table_name = person_block))]
pub struct PersonBlock {
  pub id: PersonBlockId,
  pub person_id: PersonId,
  pub target_id: PersonId,
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_block))]
pub struct PersonBlockForm {
  pub person_id: PersonId,
  pub target_id: PersonId,
}
