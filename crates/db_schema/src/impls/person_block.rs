use crate::{
  newtypes::PersonId,
  schema::{person, person_block},
  source::{
    person::Person,
    person_block::{PersonBlock, PersonBlockForm},
  },
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  result::Error,
  select,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

impl PersonBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_recipient_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      person_block::table.find((for_person_id, for_recipient_id)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::PersonIsBlocked.into())
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Person>, Error> {
    let conn = &mut get_conn(pool).await?;
    let target_person_alias = diesel::alias!(person as person1);

    person_block::table
      .inner_join(person::table.on(person_block::person_id.eq(person::id)))
      .inner_join(
        target_person_alias.on(person_block::target_id.eq(target_person_alias.field(person::id))),
      )
      .select(target_person_alias.fields(person::all_columns))
      .filter(person_block::person_id.eq(person_id))
      .filter(target_person_alias.field(person::deleted).eq(false))
      .order_by(person_block::published)
      .load::<Person>(conn)
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
    insert_into(person_block::table)
      .values(person_block_form)
      .on_conflict((person_block::person_id, person_block::target_id))
      .do_update()
      .set(person_block_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(pool: &mut DbPool<'_>, person_block_form: &Self::Form) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      person_block::table.find((person_block_form.person_id, person_block_form.target_id)),
    )
    .execute(conn)
    .await
  }
}
