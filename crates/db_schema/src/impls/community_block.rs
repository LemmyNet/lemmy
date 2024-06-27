use crate::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_block},
  source::{
    community::Community,
    community_block::{CommunityBlock, CommunityBlockForm},
  },
  traits::Blockable,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into},
  result::Error,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl CommunityBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_community_id: CommunityId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_block::table.find((for_person_id, for_community_id)),
    ))
    .get_result(conn)
    .await
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Community>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_block::table
      .inner_join(community::table)
      .select(community::all_columns)
      .filter(community_block::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_block::published)
      .load::<Community>(conn)
      .await
  }
}

#[async_trait]
impl Blockable for CommunityBlock {
  type Form = CommunityBlockForm;
  async fn block(pool: &mut DbPool<'_>, community_block_form: &Self::Form) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_block::table)
      .values(community_block_form)
      .on_conflict((community_block::person_id, community_block::community_id))
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
    diesel::delete(community_block::table.find((
      community_block_form.person_id,
      community_block_form.community_id,
    )))
    .execute(conn)
    .await
  }
}
