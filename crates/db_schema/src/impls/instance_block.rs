use crate::{
  newtypes::{InstanceId, PersonId},
  schema::instance_actions,
  source::instance_block::{InstanceBlock, InstanceBlockForm},
  traits::Blockable,
  utils::{
    find_action,
    get_conn,
    now,
    uplete::{OrDelete, UpleteCount},
    DbPool,
  },
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{self, exists, insert_into},
  result::Error,
  select,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl InstanceBlock {
  fn as_select_unwrap() -> (
    instance_actions::person_id,
    instance_actions::instance_id,
    dsl::AssumeNotNull<instance_actions::blocked>,
  ) {
    (
      instance_actions::person_id,
      instance_actions::instance_id,
      instance_actions::blocked.assume_not_null(),
    )
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_instance_id: InstanceId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(find_action(
      instance_actions::blocked,
      (for_person_id, for_instance_id),
    )))
    .get_result(conn)
    .await
  }
}

#[async_trait]
impl Blockable for InstanceBlock {
  type Form = InstanceBlockForm;
  async fn block(pool: &mut DbPool<'_>, instance_block_form: &Self::Form) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let instance_block_form = (
      instance_block_form,
      instance_actions::blocked.eq(now().nullable()),
    );
    insert_into(instance_actions::table)
      .values(instance_block_form)
      .on_conflict((instance_actions::person_id, instance_actions::instance_id))
      .do_update()
      .set(instance_block_form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    instance_block_form: &Self::Form,
  ) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(instance_actions::table.find((
      instance_block_form.person_id,
      instance_block_form.instance_id,
    )))
    .set(instance_actions::blocked.eq(None::<DateTime<Utc>>))
    .or_delete()
    .get_result(conn)
    .await
  }
}
