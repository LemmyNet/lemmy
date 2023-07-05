use crate::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  diesel::BoolExpressionMethods,
  newtypes::{PersonId, PostId},
  schema::person_post_aggregates::dsl::{person_id, person_post_aggregates, post_id},
  utils::{DbPool, DbPoolRef, RunQueryDsl},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};

impl PersonPostAggregates {
  pub async fn upsert(pool: DbPoolRef<'_>, form: &PersonPostAggregatesForm) -> Result<Self, Error> {
    let conn = pool;
    insert_into(person_post_aggregates)
      .values(form)
      .on_conflict((person_id, post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(
    pool: DbPoolRef<'_>,
    person_id_: PersonId,
    post_id_: PostId,
  ) -> Result<Self, Error> {
    let conn = pool;
    person_post_aggregates
      .filter(post_id.eq(post_id_).and(person_id.eq(person_id_)))
      .first::<Self>(conn)
      .await
  }
}
