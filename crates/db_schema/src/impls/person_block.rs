use crate::{
  newtypes::PersonId,
  schema::person_actions,
  source::person_block::{PersonBlock, PersonBlockForm},
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
  dsl,
  dsl::{exists, insert_into},
  result::Error,
  select,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl PersonBlock {
  fn as_select_unwrap() -> (
    person_actions::person_id,
    person_actions::target_id,
    dsl::AssumeNotNull<person_actions::blocked>,
  ) {
    (
      person_actions::person_id,
      person_actions::target_id,
      person_actions::blocked.assume_not_null(),
    )
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_recipient_id: PersonId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(find_action(
      person_actions::blocked,
      (for_person_id, for_recipient_id),
    )))
    .get_result(conn)
    .await
  }
}

#[async_trait]
impl Blockable for PersonBlock {
  type Form = PersonBlockForm;
  async fn block(
    pool: &mut DbPool<'_>,
    person_block_form: &PersonBlockForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_block_form = (
      person_block_form,
      person_actions::blocked.eq(now().nullable()),
    );
    insert_into(person_actions::table)
      .values(person_block_form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(person_block_form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(pool: &mut DbPool<'_>, person_block_form: &Self::Form) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_actions::table.find((person_block_form.person_id, person_block_form.target_id)),
    )
    .set(person_actions::blocked.eq(None::<DateTime<Utc>>))
    .or_delete()
    .get_result(conn)
    .await
  }
}
