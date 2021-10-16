use crate::{
  newtypes::{PersonBlockId, PersonId},
  schema::person_block,
};
use serde::Serialize;

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person_block"]
pub struct PersonBlock {
  pub id: PersonBlockId,
  pub person_id: PersonId,
  pub target_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "person_block"]
pub struct PersonBlockForm {
  pub person_id: PersonId,
  pub target_id: PersonId,
}
