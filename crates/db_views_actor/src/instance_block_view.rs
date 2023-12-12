use crate::structs::InstanceBlockView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{instance, instance_block, person, site},
  utils::{get_conn, DbPool},
};

impl InstanceBlockView {
  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance_block::table
      .inner_join(person::table)
      .inner_join(instance::table)
      .left_join(site::table.on(site::instance_id.eq(instance::id)))
      .select((
        person::all_columns,
        instance::all_columns,
        site::all_columns.nullable(),
      ))
      .filter(instance_block::person_id.eq(person_id))
      .order_by(instance_block::published)
      .load::<InstanceBlockView>(conn)
      .await
  }
}
