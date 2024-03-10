use crate::structs::PersonBlockView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{person, person_actions},
  utils::{action_query, get_conn, DbPool},
};

impl PersonBlockView {
  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let target_person_alias = diesel::alias!(person as person1);

    action_query(person_actions::blocked)
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .inner_join(
        target_person_alias.on(person_actions::target_id.eq(target_person_alias.field(person::id))),
      )
      .select((
        person::all_columns,
        target_person_alias.fields(person::all_columns),
      ))
      .filter(person_actions::person_id.eq(person_id))
      .filter(target_person_alias.field(person::deleted).eq(false))
      .order_by(person_actions::blocked)
      .load::<PersonBlockView>(conn)
      .await
  }
}
