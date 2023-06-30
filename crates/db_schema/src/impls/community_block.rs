use crate::{
  schema::community_block::dsl::{community_block, community_id, person_id},
  source::community_block::{CommunityBlock, CommunityBlockForm},
  traits::Blockable,
  utils::DbConn,
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Blockable for CommunityBlock {
  type Form = CommunityBlockForm;
  async fn block(conn: &mut DbConn, community_block_form: &Self::Form) -> Result<Self, Error> {
    insert_into(community_block)
      .values(community_block_form)
      .on_conflict((person_id, community_id))
      .do_update()
      .set(community_block_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(conn: &mut DbConn, community_block_form: &Self::Form) -> Result<usize, Error> {
    diesel::delete(
      community_block
        .filter(person_id.eq(community_block_form.person_id))
        .filter(community_id.eq(community_block_form.community_id)),
    )
    .execute(conn)
    .await
  }
}
