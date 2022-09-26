use crate::{
  source::community_block::{CommunityBlock, CommunityBlockForm},
  traits::Blockable,
};
use diesel::{dsl::*, result::Error, *};

impl Blockable for CommunityBlock {
  type Form = CommunityBlockForm;
  fn block(conn: &mut PgConnection, community_block_form: &Self::Form) -> Result<Self, Error> {
    use crate::schema::community_block::dsl::*;
    insert_into(community_block)
      .values(community_block_form)
      .on_conflict((person_id, community_id))
      .do_update()
      .set(community_block_form)
      .get_result::<Self>(conn)
  }
  fn unblock(conn: &mut PgConnection, community_block_form: &Self::Form) -> Result<usize, Error> {
    use crate::schema::community_block::dsl::*;
    diesel::delete(
      community_block
        .filter(person_id.eq(community_block_form.person_id))
        .filter(community_id.eq(community_block_form.community_id)),
    )
    .execute(conn)
  }
}
