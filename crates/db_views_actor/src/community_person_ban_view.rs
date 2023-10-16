use crate::structs::CommunityPersonBanView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::community_person_ban,
  utils::{get_conn, DbPool},
};

impl CommunityPersonBanView {
  pub async fn get(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    from_community_id: CommunityId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_person_ban::table
        .filter(community_person_ban::community_id.eq(from_community_id))
        .filter(community_person_ban::person_id.eq(from_person_id)),
    ))
    .get_result::<bool>(conn)
    .await
  }
}
