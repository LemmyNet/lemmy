use crate::{
  newtypes::PersonId,
  schema::{person, person_actions},
  source::{
    person::Person,
    person_block::{PersonBlock, PersonBlockForm},
  },
  traits::Blockable,
  utils::{get_conn, now, uplete, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  expression::SelectableHelper,
  result::Error,
  select,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

impl PersonBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_recipient_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = person_actions::table
      .find((for_person_id, for_recipient_id))
      .filter(person_actions::blocked.is_not_null());
    select(not(exists(find_action)))
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

    person_actions::table
      .filter(person_actions::blocked.is_not_null())
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .inner_join(
        target_person_alias.on(person_actions::target_id.eq(target_person_alias.field(person::id))),
      )
      .select(target_person_alias.fields(person::all_columns))
      .filter(person_actions::person_id.eq(person_id))
      .filter(target_person_alias.field(person::deleted).eq(false))
      .order_by(person_actions::blocked)
      .load::<Person>(conn)
      .await
  }
}

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
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    person_block_form: &Self::Form,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(
      person_actions::table.find((person_block_form.person_id, person_block_form.target_id)),
    )
    .set_null(person_actions::blocked)
    .get_result(conn)
    .await
  }
}
