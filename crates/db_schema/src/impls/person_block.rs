use crate::{
  newtypes::PersonId,
  source::person_block::{PersonBlock, PersonBlockForm},
  traits::Blockable,
};
use diesel::{dsl::*, result::Error, *};

impl PersonBlock {
  pub fn read(
    conn: &mut PgConnection,
    for_person_id: PersonId,
    for_recipient_id: PersonId,
  ) -> Result<Self, Error> {
    use crate::schema::person_block::dsl::*;
    person_block
      .filter(person_id.eq(for_person_id))
      .filter(target_id.eq(for_recipient_id))
      .first::<Self>(conn)
  }
}

impl Blockable for PersonBlock {
  type Form = PersonBlockForm;
  fn block(conn: &mut PgConnection, person_block_form: &PersonBlockForm) -> Result<Self, Error> {
    use crate::schema::person_block::dsl::*;
    insert_into(person_block)
      .values(person_block_form)
      .on_conflict((person_id, target_id))
      .do_update()
      .set(person_block_form)
      .get_result::<Self>(conn)
  }
  fn unblock(conn: &mut PgConnection, person_block_form: &Self::Form) -> Result<usize, Error> {
    use crate::schema::person_block::dsl::*;
    diesel::delete(
      person_block
        .filter(person_id.eq(person_block_form.person_id))
        .filter(target_id.eq(person_block_form.target_id)),
    )
    .execute(conn)
  }
}
