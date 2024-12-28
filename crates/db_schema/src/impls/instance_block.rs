use crate::{
  newtypes::{InstanceId, PersonId},
  schema::{instance, instance_actions},
  source::{
    instance::Instance,
    instance_block::{InstanceBlock, InstanceBlockForm},
  },
  traits::Blockable,
  utils::{action_query, find_action, get_conn, now, uplete, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  expression::SelectableHelper,
  result::Error,
  select,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

impl InstanceBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_instance_id: InstanceId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(find_action(
      instance_actions::blocked,
      (for_person_id, for_instance_id),
    ))))
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
    action_query(instance_actions::blocked)
      .inner_join(instance::table)
      .select(instance::all_columns)
      .filter(instance_actions::person_id.eq(person_id))
      .order_by(instance_actions::blocked)
      .load::<Instance>(conn)
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
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    instance_block_form: &Self::Form,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(instance_actions::table.find((
      instance_block_form.person_id,
      instance_block_form.instance_id,
    )))
    .set_null(instance_actions::blocked)
    .get_result(conn)
    .await
  }
}
