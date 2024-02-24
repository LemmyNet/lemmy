use crate::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  newtypes::{PersonId, PostId},
  schema::post_actions,
  utils::{get_conn, now, DbPool},
};
use diesel::{
  dsl,
  insert_into,
  result::Error,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl PersonPostAggregates {
  fn as_select_unwrap() -> (
    post_actions::person_id,
    post_actions::post_id,
    dsl::AssumeNotNull<post_actions::read_comments_amount>,
    dsl::AssumeNotNull<post_actions::read_comments>,
  ) {
    (
      post_actions::person_id,
      post_actions::post_id,
      post_actions::read_comments_amount.assume_not_null(),
      post_actions::read_comments.assume_not_null(),
    )
  }

  pub async fn upsert(
    pool: &mut DbPool<'_>,
    form: &PersonPostAggregatesForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let form = (form, post_actions::read_comments.eq(now().nullable()));
    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id_: PersonId,
    post_id_: PostId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    post_actions::table
      .find((person_id_, post_id_))
      .filter(post_actions::read_comments.is_not_null())
      .filter(post_actions::read_comments_amount.is_not_null())
      .select(Self::as_select_unwrap())
      .first::<Self>(conn)
      .await
  }
}
