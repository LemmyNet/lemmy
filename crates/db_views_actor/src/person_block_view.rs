use crate::structs::PersonBlockView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{person, person_block},
  source::person::{Person, PersonSafe},
  traits::{ToSafe, ViewToVec},
  utils::{get_conn, DbPool},
};

type PersonBlockViewTuple = (PersonSafe, PersonSafe);

impl PersonBlockView {
  pub async fn for_person(pool: &DbPool, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(person as person1);

    let res = person_block::table
      .inner_join(person::table)
      .inner_join(person_alias_1)
      .select((
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
      ))
      .filter(person_block::person_id.eq(person_id))
      .filter(person_alias_1.field(person::deleted).eq(false))
      .order_by(person_block::published)
      .load::<PersonBlockViewTuple>(conn)
      .await?;

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
