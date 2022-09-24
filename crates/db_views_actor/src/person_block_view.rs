use crate::structs::PersonBlockView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{person, person_block},
  source::person::{Person, PersonSafe},
  traits::{ToSafe, ViewToVec},
};

type PersonBlockViewTuple = (PersonSafe, PersonSafe);

impl PersonBlockView {
  pub fn for_person(conn: &mut PgConnection, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let person_alias_1 = diesel::alias!(person as person1);

    let res = person_block::table
      .inner_join(person::table)
      .inner_join(person_alias_1)
      .select((
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
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
