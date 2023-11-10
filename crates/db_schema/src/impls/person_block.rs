use crate::{
  newtypes::PersonId,
  schema::person_block::dsl::{person_block, person_id, target_id},
  source::person_block::{PersonBlock, PersonBlockForm},
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into},
  result::Error,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl PersonBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_recipient_id: PersonId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      person_block
        .filter(person_id.eq(for_person_id))
        .filter(target_id.eq(for_recipient_id)),
    ))
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
    insert_into(person_block)
      .values(person_block_form)
      .on_conflict((person_id, target_id))
      .do_update()
      .set(person_block_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(pool: &mut DbPool<'_>, person_block_form: &Self::Form) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      person_block
        .filter(person_id.eq(person_block_form.person_id))
        .filter(target_id.eq(person_block_form.target_id)),
    )
    .execute(conn)
    .await
  }
}
