use crate::{
  newtypes::{CommunityId, PersonId},
  schema::community_actions,
  source::community_block::{CommunityBlock, CommunityBlockForm},
  traits::Blockable,
  utils::{get_conn, now, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{self, exists, insert_into},
  result::Error,
  select,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl CommunityBlock {
  fn as_select_unwrap() -> (
    community_actions::person_id,
    community_actions::community_id,
    dsl::AssumeNotNull<community_actions::blocked>,
  ) {
    (
      community_actions::person_id,
      community_actions::community_id,
      community_actions::blocked.assume_not_null(),
    )
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_community_id: CommunityId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_actions::table
        .find((for_person_id, for_community_id))
        .filter(community_actions::blocked.is_not_null()),
    ))
    .get_result(conn)
    .await
  }
}

#[async_trait]
impl Blockable for CommunityBlock {
  type Form = CommunityBlockForm;
  async fn block(pool: &mut DbPool<'_>, community_block_form: &Self::Form) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let community_block_form = (
      community_block_form,
      community_actions::blocked.eq(now().nullable()),
    );
    insert_into(community_actions::table)
      .values(community_block_form)
      .on_conflict((
        community_actions::person_id,
        community_actions::community_id,
      ))
      .do_update()
      .set(community_block_form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    community_block_form: &Self::Form,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(community_actions::table.find((
      community_block_form.person_id,
      community_block_form.community_id,
    )))
    .set(community_actions::blocked.eq(None::<DateTime<Utc>>))
    .execute(conn)
    .await
  }
}
