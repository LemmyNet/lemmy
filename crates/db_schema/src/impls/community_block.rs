use crate::{
    schema::community_block::dsl::{community_block, community_id, person_id},
    source::community_block::{CommunityBlock, CommunityBlockForm},
    traits::Blockable,
    utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Blockable for CommunityBlock {
    type Form = CommunityBlockForm;
    async fn block(
        pool: &mut DbPool<'_>,
        community_block_form: &Self::Form,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(community_block)
            .values(community_block_form)
            .on_conflict((person_id, community_id))
            .do_update()
            .set(community_block_form)
            .get_result::<Self>(conn)
            .await
    }
    async fn unblock(
        pool: &mut DbPool<'_>,
        community_block_form: &Self::Form,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            community_block
                .filter(person_id.eq(community_block_form.person_id))
                .filter(community_id.eq(community_block_form.community_id)),
        )
        .execute(conn)
        .await
    }
}
