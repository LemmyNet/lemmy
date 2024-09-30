use crate::structs::CommunityPersonBanView;
use diesel::{
  dsl::{exists, not},
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::community_person_ban,
  utils::{get_conn, DbPool},
};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

impl CommunityPersonBanView {
  pub async fn check(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    from_community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      community_person_ban::table
        .filter(community_person_ban::community_id.eq(from_community_id))
        .filter(community_person_ban::person_id.eq(from_person_id)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::PersonIsBannedFromCommunity.into())
  }
}
