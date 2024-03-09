use crate::structs::CommunityPersonBanView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::community_actions,
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
      community_actions::table
        .find((from_person_id, from_community_id))
        .filter(community_actions::received_ban.is_not_null()),
    ))
    .get_result::<bool>(conn)
    .await
  }
}
