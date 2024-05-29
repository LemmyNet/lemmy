use crate::{
  newtypes::{CommunityId, PersonId},
  schema::community_actions,
  source::community_block::{CommunityBlock, CommunityBlockForm},
  traits::Blockable,
  utils::{
    find_action,
    get_conn,
    now,
    uplete::{uplete, UpleteCount},
    DbPool,
  },
};
use diesel::{
  dsl::{exists, insert_into},
  expression::SelectableHelper,
  result::Error,
  select,
  ExpressionMethods,
  NullableExpressionMethods,
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
    select(exists(find_action(
      community_actions::blocked,
      (for_person_id, for_community_id),
    )))
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
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    community_block_form: &Self::Form,
  ) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete(community_actions::table.find((
      community_block_form.person_id,
      community_block_form.community_id,
    )))
    .set_null(community_actions::blocked)
    .get_result(conn)
    .await
  }
}
