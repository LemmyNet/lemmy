use crate::structs::PersonBlockView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{person, person_block},
  source::person::Person,
  traits::JoinView,
  utils::{get_conn, DbPool},
};

type PersonBlockViewTuple = (Person, Person);

impl PersonBlockView {
  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let target_person_alias = diesel::alias!(person as person1);

    let res = person_block::table
      .inner_join(person::table.on(person_block::person_id.eq(person::id)))
      .inner_join(
        target_person_alias.on(person_block::target_id.eq(target_person_alias.field(person::id))),
      )
      .select((
        person::all_columns,
        target_person_alias.fields(person::all_columns),
      ))
      .filter(person_block::person_id.eq(person_id))
      .filter(target_person_alias.field(person::deleted).eq(false))
      .order_by(person_block::published)
      .load::<PersonBlockViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for PersonBlockView {
  type JoinTuple = PersonBlockViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      person: a.0,
      target: a.1,
    }
  }
}
