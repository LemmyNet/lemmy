use crate::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  newtypes::{PersonId, PostId},
  schema::person_post_aggregates::dsl::{person_id, person_post_aggregates, post_id},
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl PersonPostAggregates {
  pub async fn upsert(
    pool: &mut DbPool<'_>,
    form: &PersonPostAggregatesForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_post_aggregates)
      .values(form)
      .on_conflict((person_id, post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id_: PersonId,
    post_id_: PostId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    person_post_aggregates
      .find((person_id_, post_id_))
      .first::<Self>(conn)
      .await
  }
}
