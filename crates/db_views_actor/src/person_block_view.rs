use crate::structs::PersonBlockView;
use diesel::{
  result::{Error, Error::QueryBuilderError},
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{person, person_block},
  utils::ActualDbPool,
};
use std::ops::DerefMut;

impl PersonBlockView {
  pub async fn for_person(pool: &ActualDbPool, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let mut conn = pool.get().await.map_err(|e| QueryBuilderError(e.into()))?;
    let target_person_alias = diesel::alias!(person as person1);

    person_block::table
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
      .load::<PersonBlockView>(conn.deref_mut())
      .await
  }
}
