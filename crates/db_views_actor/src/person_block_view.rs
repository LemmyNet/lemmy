use crate::structs::PersonBlockView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{person, person_alias_1, person_block},
  source::person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  traits::{ToSafe, ViewToVec},
};

type PersonBlockViewTuple = (PersonSafe, PersonSafeAlias1);

impl PersonBlockView {
  pub fn for_person(conn: &PgConnection, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let res = person_block::table
      .inner_join(person::table)
      .inner_join(person_alias_1::table) // TODO I dont know if this will be smart abt the column
      .select((
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .filter(person_block::person_id.eq(person_id))
      .order_by(person_block::published)
      .load::<PersonBlockViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PersonBlockView {
  type DbTuple = PersonBlockViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        person: a.0,
        target: a.1,
      })
      .collect::<Vec<Self>>()
  }
}
