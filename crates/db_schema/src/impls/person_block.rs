use crate::{
  newtypes::PersonId,
  schema::person_block::dsl::{person_block, person_id, target_id},
  source::person_block::{PersonBlock, PersonBlockForm},
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  result::Error,
  select,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

impl PersonBlock {
  pub async fn check(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_recipient_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      person_block.find((for_person_id, for_recipient_id)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::PersonIsBlocked.into())
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
    diesel::delete(person_block.find((person_block_form.person_id, person_block_form.target_id)))
      .execute(conn)
      .await
  }
}
