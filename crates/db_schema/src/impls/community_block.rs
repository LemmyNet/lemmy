use crate::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_actions},
  source::{
    community::Community,
    community_block::{CommunityBlock, CommunityBlockForm},
  },
  traits::Blockable,
  utils::{get_conn, now, uplete, DbPool},
};
use diesel::{
  dsl::{exists, insert_into, not},
  expression::SelectableHelper,
  result::Error,
  select,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

impl CommunityBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .find((for_person_id, for_community_id))
      .filter(community_actions::blocked.is_not_null());
    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(LemmyErrorType::CommunityIsBlocked.into())
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Community>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_actions::table
      .filter(community_actions::blocked.is_not_null())
      .inner_join(community::table)
      .select(community::all_columns)
      .filter(community_actions::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_actions::blocked)
      .load::<Community>(conn)
      .await
  }
}

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
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((
      community_block_form.person_id,
      community_block_form.community_id,
    )))
    .set_null(community_actions::blocked)
    .get_result(conn)
    .await
  }
}
