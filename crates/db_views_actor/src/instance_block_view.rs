use crate::structs::InstanceBlockView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{instance, instance_block, person, site},
  source::{instance::Instance, person::Person, site::Site},
  traits::JoinView,
  utils::{get_conn, DbPool},
};

type InstanceBlockViewTuple = (Person, Instance, Option<Site>);

impl InstanceBlockView {
  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = instance_block::table
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
      .load::<InstanceBlockViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for InstanceBlockView {
  type JoinTuple = InstanceBlockViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      person: a.0,
      instance: a.1,
      site: a.2,
    }
  }
}
