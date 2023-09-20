use crate::{
  schema::instance_block::dsl::{instance_block, instance_id, person_id},
  source::instance_block::{InstanceBlock, InstanceBlockForm},
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Blockable for InstanceBlock {
  type Form = InstanceBlockForm;
  async fn block(pool: &mut DbPool<'_>, instance_block_form: &Self::Form) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(instance_block)
      .values(instance_block_form)
      .on_conflict((person_id, instance_id))
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
    diesel::delete(
      instance_block
        .filter(person_id.eq(instance_block_form.person_id))
        .filter(instance_id.eq(instance_block_form.instance_id)),
    )
    .execute(conn)
    .await
  }
}
