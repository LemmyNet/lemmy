use crate::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  newtypes::{PersonId, PostId},
};
use diesel::{result::Error, *};

impl PersonPostAggregates {
  pub fn upsert(conn: &mut PgConnection, form: &PersonPostAggregatesForm) -> Result<Self, Error> {
    use crate::schema::person_post_aggregates::dsl::*;
    insert_into(person_post_aggregates)
      .values(form)
      .on_conflict((person_id, post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
  }
  pub fn read(
    conn: &mut PgConnection,
    person_id_: PersonId,
    post_id_: PostId,
  ) -> Result<Self, Error> {
    use crate::schema::person_post_aggregates::dsl::*;
    person_post_aggregates
      .filter(post_id.eq(post_id_).and(person_id.eq(person_id_)))
      .first::<Self>(conn)
  }
}
