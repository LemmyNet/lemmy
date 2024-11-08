use crate::{
  newtypes::{InstanceId, PersonId},
  schema::{instance, instance_block},
  source::{
    instance::Instance,
    instance_block::{InstanceBlock, InstanceBlockForm},
  },
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  result::Error,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

impl InstanceBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_instance_id: InstanceId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      instance_block::table.find((for_person_id, for_instance_id)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::InstanceIsBlocked.into())
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Instance>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance_block::table
      .inner_join(instance::table)
      .select(instance::all_columns)
      .filter(instance_block::person_id.eq(person_id))
      .order_by(instance_block::published)
      .load::<Instance>(conn)
      .await
  }
}

#[async_trait]
impl Blockable for InstanceBlock {
  type Form = InstanceBlockForm;
  async fn block(pool: &mut DbPool<'_>, instance_block_form: &Self::Form) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(instance_block::table)
      .values(instance_block_form)
      .on_conflict((instance_block::person_id, instance_block::instance_id))
      .do_update()
      .set(instance_block_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    instance_block_form: &Self::Form,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance_block::table.find((
      instance_block_form.person_id,
      instance_block_form.instance_id,
    )))
    .execute(conn)
    .await
  }
}
