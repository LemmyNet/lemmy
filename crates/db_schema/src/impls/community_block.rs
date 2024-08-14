use crate::{
  newtypes::{CommunityId, PersonId},
  schema::community_block::dsl::{community_block, community_id, person_id},
  source::community_block::{CommunityBlock, CommunityBlockForm},
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  result::Error,
  select,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

impl CommunityBlock {
  pub async fn check(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      community_block.find((for_person_id, for_community_id)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::CommunityIsBlocked.into())
  }
}

#[async_trait]
impl Blockable for CommunityBlock {
  type Form = CommunityBlockForm;
  async fn block(pool: &mut DbPool<'_>, community_block_form: &Self::Form) -> Result<Self, Error> {
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
    diesel::delete(community_block.find((
      community_block_form.person_id,
      community_block_form.community_id,
    )))
    .execute(conn)
    .await
  }
}
